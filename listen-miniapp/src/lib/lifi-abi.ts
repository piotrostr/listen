// Diamond proxy ABI for LiFi
export const LIFI_DIAMOND_ABI = [
  {
    inputs: [
      { internalType: "address", name: "_contractOwner", type: "address" },
      { internalType: "address", name: "_diamondCutFacet", type: "address" },
    ],
    stateMutability: "payable",
    type: "constructor",
  },
  { inputs: [], name: "CalldataEmptyButInitNotZero", type: "error" },
  { inputs: [], name: "FacetAddressIsNotZero", type: "error" },
  { inputs: [], name: "FacetAddressIsZero", type: "error" },
  { inputs: [], name: "FacetContainsNoCode", type: "error" },
  { inputs: [], name: "FunctionAlreadyExists", type: "error" },
  { inputs: [], name: "FunctionDoesNotExist", type: "error" },
  { inputs: [], name: "FunctionIsImmutable", type: "error" },
  { inputs: [], name: "IncorrectFacetCutAction", type: "error" },
  { inputs: [], name: "InitReverted", type: "error" },
  { inputs: [], name: "InitZeroButCalldataNotEmpty", type: "error" },
  { inputs: [], name: "NoSelectorsInFace", type: "error" },
  { stateMutability: "payable", type: "fallback" },
  { stateMutability: "payable", type: "receive" },
];

export const PERMIT2_PROXY_ABI = [
  {
    inputs: [
      {
        internalType: "bytes",
        name: "_diamondCalldata",
        type: "bytes",
      },
      {
        components: [
          {
            components: [
              {
                internalType: "address",
                name: "token",
                type: "address",
              },
              {
                internalType: "uint256",
                name: "amount",
                type: "uint256",
              },
            ],
            internalType: "struct ISignatureTransfer.TokenPermissions",
            name: "permitted",
            type: "tuple",
          },
          {
            internalType: "uint256",
            name: "nonce",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "deadline",
            type: "uint256",
          },
        ],
        internalType: "struct ISignatureTransfer.PermitTransferFrom",
        name: "_permit",
        type: "tuple",
      },
      {
        internalType: "bytes",
        name: "_signature",
        type: "bytes",
      },
    ],
    name: "callDiamondWithPermit2",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
];
