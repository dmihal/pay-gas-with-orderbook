contract;

struct HelloWorld {}

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        log(HelloWorld {});
        true
    }
}
