use argon2::Params;
use minidb_utils::Argon2Params;

#[test]
fn test_argon2params() {
    let my_params = Argon2Params {
        iterations: 1,
        memory: 1024,
        parallelism: 1,
        output_len: Some(32),
    };

    let params: Params = my_params
        .clone()
        .try_into()
        .expect("Failed to convert Argon2Params to argon2::Params");

    assert_eq!(params.m_cost(), my_params.memory);
    assert_eq!(params.t_cost(), my_params.iterations);
    assert_eq!(params.p_cost(), my_params.parallelism);
    assert_eq!(params.output_len(), my_params.output_len);
}
