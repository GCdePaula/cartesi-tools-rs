pub use alloy_primitives;
pub use alloy_sol_types;

use alloy_sol_types::sol;

sol! {
    /// @notice An advance request from an EVM-compatible blockchain to a Cartesi Machine.
    /// @param chainId The chain ID
    /// @param appContract The application contract address
    /// @param msgSender The address of whoever sent the input
    /// @param blockNumber The number of the block in which the input was added
    /// @param blockTimestamp The timestamp of the block in which the input was added
    /// @param prevRandao The latest RANDAO mix of the post beacon state of the previous block
    /// @param index The index of the input in the input box
    /// @param payload The payload provided by the message sender
    /// @dev See EIP-4399 for safe usage of `prevRandao`.
    #[derive(Debug, PartialEq, Eq)]
    function EvmAdvance(
        uint256 chainId,
        address appContract,
        address msgSender,
        uint256 blockNumber,
        uint256 blockTimestamp,
        uint256 prevRandao,
        uint256 index,
        bytes calldata payload
    ) external;

    /// @notice A piece of verifiable information.
    /// @param payload An arbitrary payload.
    #[derive(Debug, PartialEq, Eq)]
    function Notice(bytes calldata payload) external;

    /// @notice A single-use permission to execute a specific message call
    /// from the context of the application contract.
    /// @param destination The address that will be called
    /// @param value The amount of Wei to be transferred through the call
    /// @param payload The payload, which—in the case of Solidity
    /// contracts—encodes a function call
    #[derive(Debug, PartialEq, Eq)]
    function Voucher(
        address destination,
        uint256 value,
        bytes calldata payload
    ) external;

    /// @notice A single-use permission to execute a specific delegate call
    /// from the context of the application contract.
    /// @param destination The address that will be called
    /// @param payload The payload, which—in the case of Solidity
    /// libraries—encodes a function call
    function DelegateCallVoucher(address destination, bytes calldata payload) external;

    /// @notice Encode an Ether deposit.
    /// @param sender The Ether sender
    /// @param value The amount of Wei being sent
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input payload
    #[derive(Debug, PartialEq, Eq)]
    function encodeEtherDeposit(
        address sender,
        uint256 value,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            sender, //              20B
            value, //               32B
            execLayerData //        arbitrary size
        );
    }

    /// @notice Encode an ERC-20 token deposit.
    /// @param token The token contract
    /// @param sender The token sender
    /// @param value The amount of tokens being sent
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input payload
    function encodeERC20Deposit(
        address token,
        address sender,
        uint256 value,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            token, //               20B
            sender, //              20B
            value, //               32B
            execLayerData //        arbitrary size
        );
    }

    /// @notice Encode an ERC-721 token deposit.
    /// @param token The token contract
    /// @param sender The token sender
    /// @param tokenId The token identifier
    /// @param baseLayerData Additional data to be interpreted by the base layer
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input payload
    /// @dev `baseLayerData` should be forwarded to `token`.
    function encodeERC721Deposit(
        address token,
        address sender,
        uint256 tokenId,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        bytes memory data = abi.encode(baseLayerData, execLayerData);
        return abi.encodePacked(
            token, //               20B
            sender, //              20B
            tokenId, //             32B
            data //                 arbitrary size
        );
    }

    /// @notice Encode an ERC-1155 single token deposit.
    /// @param token The ERC-1155 token contract
    /// @param sender The token sender
    /// @param tokenId The identifier of the token being transferred
    /// @param value Transfer amount
    /// @param baseLayerData Additional data to be interpreted by the base layer
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input payload
    /// @dev `baseLayerData` should be forwarded to `token`.
    function encodeSingleERC1155Deposit(
        address token,
        address sender,
        uint256 tokenId,
        uint256 value,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        bytes memory data = abi.encode(baseLayerData, execLayerData);
        return abi.encodePacked(
            token, //               20B
            sender, //              20B
            tokenId, //             32B
            value, //               32B
            data //                 arbitrary size
        );
    }

    /// @notice Encode an ERC-1155 batch token deposit.
    /// @param token The ERC-1155 token contract
    /// @param sender The token sender
    /// @param tokenIds The identifiers of the tokens being transferred
    /// @param values Transfer amounts per token type
    /// @param baseLayerData Additional data to be interpreted by the base layer
    /// @param execLayerData Additional data to be interpreted by the execution layer
    /// @return The encoded input payload
    /// @dev `baseLayerData` should be forwarded to `token`.
    function encodeBatchERC1155Deposit(
        address token,
        address sender,
        uint256[] calldata tokenIds,
        uint256[] calldata values,
        bytes calldata baseLayerData,
        bytes calldata execLayerData
    ) internal pure returns (bytes memory) {
        bytes memory data = abi.encode(tokenIds, values, baseLayerData, execLayerData);
        return abi.encodePacked(
            token, //                   20B
            sender, //                  20B
            data //                     arbitrary size
        );
    }
}

pub type Input = EvmAdvanceCall;
pub type Voucher = VoucherCall;
pub type Notice = NoticeCall;
