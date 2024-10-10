use std::fmt::Debug;

use arith::{Field, FieldSerde};
use mpi::{
    environment::Universe,
    ffi,
    topology::{Process, SimpleCommunicator},
    traits::*,
};
use transcript::{FiatShamirHash, Transcript, TranscriptInstance};

#[macro_export]
macro_rules! root_println {
    ($config: expr, $($arg:tt)*) => {
        if $config.is_root() {
            println!($($arg)*);
        }
    };
}

static mut UNIVERSE: Option<Universe> = None;
static mut WORLD: Option<SimpleCommunicator> = None;

#[derive(Clone)]
pub struct MPIConfig {
    pub universe: Option<&'static mpi::environment::Universe>,
    pub world: Option<&'static SimpleCommunicator>,
    pub world_size: i32,
    pub world_rank: i32,
}

impl Default for MPIConfig {
    fn default() -> Self {
        Self {
            universe: None,
            world: None,
            world_size: 1,
            world_rank: 0,
        }
    }
}

impl Debug for MPIConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let universe_fmt = if self.universe.is_none() {
            Option::<usize>::None
        } else {
            Some(self.universe.unwrap().buffer_size())
        };

        let world_fmt = if self.world.is_none() {
            Option::<usize>::None
        } else {
            Some(0usize)
        };

        f.debug_struct("MPIConfig")
            .field("universe", &universe_fmt)
            .field("world", &world_fmt)
            .field("world_size", &self.world_size)
            .field("world_rank", &self.world_rank)
            .finish()
    }
}

// Note: may not be correct
impl PartialEq for MPIConfig {
    fn eq(&self, other: &Self) -> bool {
        self.world_rank == other.world_rank && self.world_size == other.world_size
    }
}

/// MPI toolkit:
impl MPIConfig {
    const ROOT_RANK: i32 = 0;

    // OK if already initialized, mpi::initialize() will return None
    #[allow(static_mut_refs)]
    pub fn init() {
        unsafe {
            let universe = mpi::initialize();
            if universe.is_some() {
                UNIVERSE = universe;
                WORLD = Some(UNIVERSE.as_ref().unwrap().world());
            }
        }
    }

    #[inline]
    pub fn finalize() {
        unsafe { ffi::MPI_Finalize() };
    }

    #[allow(static_mut_refs)]
    pub fn new() -> Self {
        Self::init();
        let universe = unsafe { UNIVERSE.as_ref() };
        let world = unsafe { WORLD.as_ref() };
        let world_size = if let Some(world) = world {
            world.size()
        } else {
            1
        };
        let world_rank = if let Some(world) = world {
            world.rank()
        } else {
            0
        };
        Self {
            universe,
            world,
            world_size,
            world_rank,
        }
    }

    #[inline]
    pub fn new_for_verifier(world_size: i32) -> Self {
        Self {
            universe: None,
            world: None,
            world_size,
            world_rank: 0,
        }
    }

    /// Return an u8 vector sharing THE SAME MEMORY SLOT with the input.
    #[inline]
    unsafe fn elem_to_u8_bytes<V: Sized>(elem: &V, byte_size: usize) -> Vec<u8> {
        Vec::<u8>::from_raw_parts((elem as *const V) as *mut u8, byte_size, byte_size)
    }

    /// Return an u8 vector sharing THE SAME MEMORY SLOT with the input.
    #[inline]
    unsafe fn vec_to_u8_bytes<F: Field>(vec: &Vec<F>) -> Vec<u8> {
        Vec::<u8>::from_raw_parts(
            vec.as_ptr() as *mut u8,
            vec.len() * F::SIZE,
            vec.capacity() * F::SIZE,
        )
    }

    #[inline]
    pub fn gather_vec<F: Field>(&self, local_vec: &Vec<F>, global_vec: &mut Vec<F>) {
        unsafe {
            assert!(global_vec.len() >= local_vec.len() * (self.world_size as usize));
            if self.world_size == 1 {
                *global_vec = local_vec.clone()
            } else if self.world_rank == Self::ROOT_RANK {
                let local_vec_u8 = Self::vec_to_u8_bytes(local_vec);
                let mut global_vec_u8 = Self::vec_to_u8_bytes(global_vec);
                self.root_process()
                    .gather_into_root(&local_vec_u8, &mut global_vec_u8);
                local_vec_u8.leak(); // discard control of the memory
                global_vec_u8.leak();
            } else {
                let local_vec_u8 = Self::vec_to_u8_bytes(local_vec);
                self.root_process().gather_into(&local_vec_u8);
                local_vec_u8.leak();
            }
        }
    }

    /// broadcast root transcript state. incurs an additional hash if self.world_size > 1
    #[inline]
    pub fn transcript_sync_up<H: FiatShamirHash>(&self, transcript: &mut TranscriptInstance<H>) {
        if self.world_size == 1 {
        } else {
            transcript.hash_to_digest();
            self.root_process().broadcast_into(&mut transcript.digest);
        }
    }

    /// Root process broadcase a value f into all the processes
    #[inline]
    pub fn root_broadcast<F: Field>(&self, f: &mut F) {
        unsafe {
            if self.world_size == 1 {
            } else {
                let mut vec_u8 = Self::elem_to_u8_bytes(f, F::SIZE);
                self.root_process().broadcast_into(&mut vec_u8);
                vec_u8.leak();
            }
        }
    }

    /// sum up all local values
    #[inline]
    pub fn sum_vec<F: Field>(&self, local_vec: &Vec<F>) -> Vec<F> {
        if self.world_size == 1 {
            local_vec.clone()
        } else if self.world_rank == Self::ROOT_RANK {
            let mut global_vec = vec![F::ZERO; local_vec.len() * (self.world_size as usize)];
            self.gather_vec(local_vec, &mut global_vec);
            for i in 0..local_vec.len() {
                for j in 1..(self.world_size as usize) {
                    global_vec[i] = global_vec[i] + global_vec[j * local_vec.len() + i];
                }
            }
            global_vec.truncate(local_vec.len());
            global_vec
        } else {
            self.gather_vec(local_vec, &mut vec![]);
            vec![]
        }
    }

    /// coef has a length of mpi_world_size
    #[inline]
    pub fn coef_combine_vec<F: Field>(&self, local_vec: &Vec<F>, coef: &[F]) -> Vec<F> {
        if self.world_size == 1 {
            // Warning: literally, it should be coef[0] * local_vec
            // but coef[0] is always one in our use case of self.world_size = 1
            local_vec.clone()
        } else if self.world_rank == Self::ROOT_RANK {
            let mut global_vec = vec![F::ZERO; local_vec.len() * (self.world_size as usize)];
            let mut ret = vec![F::ZERO; local_vec.len()];
            self.gather_vec(local_vec, &mut global_vec);
            for i in 0..local_vec.len() {
                for j in 0..(self.world_size as usize) {
                    ret[i] += global_vec[j * local_vec.len() + i] * coef[j];
                }
            }
            ret
        } else {
            self.gather_vec(local_vec, &mut vec![]);
            vec![]
        }
    }

    #[inline(always)]
    pub fn world_size(&self) -> usize {
        self.world_size as usize
    }

    #[inline(always)]
    pub fn world_rank(&self) -> usize {
        self.world_rank as usize
    }

    #[inline(always)]
    pub fn is_root(&self) -> bool {
        self.world_rank == Self::ROOT_RANK
    }

    #[inline(always)]
    pub fn root_process(&self) -> Process {
        self.world.unwrap().process_at_rank(Self::ROOT_RANK)
    }

    /// Transcript IO for MPI
    #[inline]
    pub fn transcript_io<F, H>(&self, ps: &[F], transcript: &mut TranscriptInstance<H>) -> F
    where
        F: Field + FieldSerde,
        H: FiatShamirHash,
    {
        assert!(ps.len() == 3 || ps.len() == 4); // 3 for x, y; 4 for simd var
        for p in ps {
            transcript.append_field_element::<F>(p);
        }
        let mut r = transcript.generate_challenge::<F>();
        self.root_broadcast(&mut r);
        r
    }
}

unsafe impl Send for MPIConfig {}
