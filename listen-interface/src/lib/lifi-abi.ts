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
