import requests


if __name__ == '__main__':
    # prove
    with open('data/compiler_out/witness.txt', 'rb') as f:
        witness = f.read()
    url = 'http://127.0.0.1:3030'
    prove_headers = {
        'Content-Type': 'application/octet-stream',
        'Content-Length': str(len(witness)),
    }
    response = requests.post(url+"/prove", headers=prove_headers, data=witness)
    proof = response.content
    print(response)
    print("Proof generated successfully, length:", len(proof))
    with open('data/compiler_out/proof_http.bin', 'wb') as f:
        f.write(proof)

    # verify
    # add u64 length of witness and proof to the beginning of the file
    witness_len = len(witness).to_bytes(8, byteorder='little')
    proof_len = len(proof).to_bytes(8, byteorder='little')
    verifier_input = witness_len + proof_len + witness + proof
    verify_headers = {
        'Content-Type': 'application/octet-stream',
        'Content-Length': str(len(proof)),
    }
    response = requests.post(url+"/verify", headers=verify_headers, data=verifier_input)
    print(response)
    # check success message
    assert response.text == "success", f"Failed to verify proof: {response.text}"
    print("Proof verified successfully")
    
    # try tempered proof
    import random
    # flip a random bit
    random_byte_index = random.randint(0, len(proof) - 1)
    random_bit_index = random.randint(0, 7)
    tempered_proof = proof[:random_byte_index] + bytes([proof[random_byte_index] ^ (1 << random_bit_index)]) + proof[random_byte_index+1:]
    tempered_input = witness_len + proof_len + witness + tempered_proof
    response = requests.post(url+"/verify", headers=verify_headers, data=tempered_input)
    # check failure message
    assert response.text == "failure", f"Failed to detect tempered proof: {response.text}"
    print("Tempered proof detected successfully")

    # try prove using witness with invalid length
    tempered_witness = witness[:-1]
    prove_headers = {
        'Content-Type': 'application/octet-stream',
        'Content-Length': str(len(tempered_witness)),
    }
    response = requests.post(url+"/prove", headers=prove_headers, data=tempered_witness)
    # check 400
    assert response.status_code == 400, f"Failed to detect invalid witness length: {response.text}"
    print("Invalid witness length detected successfully")
