// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract StorageFibonacci {
    uint256 public storedData;

    function set(uint256 x) public {
        storedData = fibonacci(x);
    }

    function get() public view returns (uint256) {
        return storedData;
    }

    // Calculate Fibonacci
    // Cannot be accessed from outside the contract
    function fibonacci(uint256 x) internal pure returns (uint256) {
        if (x == 0) return 0;
        if (x == 1) return 1;

        uint256 a = 0;
        uint256 b = 1;
        uint256 c;

        for (uint256 i = 2; i <= x; i++) {
            c = a + b;
            a = b;
            b = c;
        }

        return b;
    }
}
