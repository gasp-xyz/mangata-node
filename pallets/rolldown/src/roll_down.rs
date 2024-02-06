pub use roll_down::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod roll_down {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![],
            }),
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("cancelResolutions"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("cancelResolutions"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("l2RequestId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("cancelJustified"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("counter"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("counter"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("deposit"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deposit"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("amount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getUpdateForL2"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getUpdateForL2"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct RollDown.L1Update"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("lastProcessedUpdate_origin_l1"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastProcessedUpdate_origin_l1",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("lastProcessedUpdate_origin_l2"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastProcessedUpdate_origin_l2",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("mat"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("mat"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress2"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("update_l1_from_l2"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("update_l1_from_l2"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("inputArray"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256[]"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("update_l1_from_l2_mat"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "update_l1_from_l2_mat",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("inputArray"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(128usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(128usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(128usize),
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct RollDown.L2ToL1Update",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("withdraw"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("withdraw"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("amount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("DepositAcceptedIntoQueue"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "DepositAcceptedIntoQueue",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("depositRecipient"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("amount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "DisputeResolutionAcceptedIntoQueue",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "DisputeResolutionAcceptedIntoQueue",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("requestId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("originalRequestId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("cancelJustified"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FundsWithdrawn"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("FundsWithdrawn"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("withdrawRecipient"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("amount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "L2UpdatesToRemovedAcceptedIntoQueue",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "L2UpdatesToRemovedAcceptedIntoQueue",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("l2UpdatesToRemove"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("WithdrawAcceptedIntoQueue"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "WithdrawAcceptedIntoQueue",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("withdrawRecipient"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("tokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("amount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("cancelAndCalculatedHash"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "cancelAndCalculatedHash",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("cancelHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("calculatedHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static ROLLDOWN_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\0`\x03\x81\x90U`\x02\x81\x90U`\x04U`\x01\x80T`\x01`\x01`\xA0\x1B\x03\x19\x163\x17\x90Ua\x1D:\x80a\0A`\09`\0\xF3\xFE`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\0\x9EW`\x005`\xE0\x1C\x80c\xB9\xCF\x81\xBE\x11a\0fW\x80c\xB9\xCF\x81\xBE\x14a\x01\x05W\x80c\xCA\x9B!\xAE\x14a\x01\x18W\x80c\xE1\xB8\xA05\x14a\x01WW\x80c\xF2n\xE9\xD0\x14a\x01iW\x80c\xF3\xFE\xF3\xA3\x14a\x01rW`\0\x80\xFD[\x80cG\xE7\xEF$\x14a\0\xA3W\x80ca\xBC\"\x1A\x14a\0\xB8W\x80c\x7F\xD4\xF8E\x14a\0\xD4W\x80c\x94a<\x16\x14a\0\xDDW\x80c\xB1S\x87\x06\x14a\0\xF0W[`\0\x80\xFD[a\0\xB6a\0\xB16`\x04a\x17%V[a\x01\x85V[\0[a\0\xC1`\x02T\x81V[`@Q\x90\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\0\xC1`\x03T\x81V[a\0\xB6a\0\xEB6`\x04a\x17sV[a\x02\xF3V[a\0\xF8a\x07\xEEV[`@Qa\0\xCB\x91\x90a\x19\xDBV[a\0\xB6a\x01\x136`\x04a\x1A\xBAV[a\x083V[a\x01Ba\x01&6`\x04a\x1A\xFCV[`\x05` R`\0\x90\x81R`@\x90 \x80T`\x01\x90\x91\x01T`\xFF\x16\x82V[`@\x80Q\x92\x83R\x90\x15\x15` \x83\x01R\x01a\0\xCBV[a\0\xB6a\x01e6`\x04a\x17%V[PPV[a\0\xC1`\x04T\x81V[a\0\xB6a\x01\x806`\x04a\x17%V[a\x0B\xCCV[`\x01`\x01`\xA0\x1B\x03\x82\x16a\x01\xD8W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x15`$\x82\x01RtInvalid token address`X\x1B`D\x82\x01R`d\x01[`@Q\x80\x91\x03\x90\xFD[`\0\x81\x11a\x02(W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01\x81\x90R`$\x82\x01R\x7FAmount must be greater than zero`D\x82\x01R`d\x01a\x01\xCFV[`\x02\x80T3\x91\x84\x91\x90`\0a\x02<\x83a\x1B+V[\x90\x91UPP`@\x80Q``\x80\x82\x01\x83R`\x01`\x01`\xA0\x1B\x03\x85\x81\x16\x80\x84R\x88\x82\x16` \x80\x86\x01\x82\x81R\x86\x88\x01\x8B\x81R`\x02\x80T`\0\x90\x81R`\x06\x85R\x8A\x90 \x89Q\x81T\x90\x89\x16`\x01`\x01`\xA0\x1B\x03\x19\x91\x82\x16\x17\x82U\x93Q`\x01\x82\x01\x80T\x91\x90\x99\x16\x94\x16\x93\x90\x93\x17\x90\x96UQ\x94\x01\x93\x90\x93U\x85Q\x91\x82R\x91\x81\x01\x91\x90\x91R\x92\x83\x01\x86\x90R\x90\x91\x7FT\xEED\x88\xADmc\"\x92\x93\xC6\x1F~h\x96\x8C\xD5%\xCE\x18\x8B\xEC\x1EMs\\\x90\x89\x95\xEEP2\x91\x01`@Q\x80\x91\x03\x90\xA1PPPPPV[`\x01T`\x01`\x01`\xA0\x1B\x03\x163\x14a\x03=W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\r`$\x82\x01Rl'7\xBA\x10:42\x907\xBB\xB72\xB9`\x99\x1B`D\x82\x01R`d\x01a\x01\xCFV[`\x02\x81Q\x11a\x03^W`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x01\xCF\x90a\x1BDV[`\0\x80`\0\x83Qg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03}Wa\x03}a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x03\xA6W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P`\x03T`\x04T\x91\x92P\x90[\x85Q\x85\x10\x15a\x06\xCBW`\0\x86\x86\x81Q\x81\x10a\x03\xCFWa\x03\xCFa\x1B\x85V[` \x02` \x01\x01Q\x90P\x80`\x01\x03a\x04yW\x86Qa\x03\xEE\x87`\x02a\x1B\x9BV[\x10a\x03\xFBWa\x03\xFBa\x1B\xAEV[`\0\x87a\x04\t\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x04\x19Wa\x04\x19a\x1B\x85V[` \x02` \x01\x01Q\x90P`\x03T\x81\x11\x15a\x04sW\x83\x81\x11\x15a\x049W\x80\x93P[\x80\x85\x87\x81Q\x81\x10a\x04LWa\x04La\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R\x85a\x04a\x81a\x1B+V[\x96Pa\x04p\x90P`\x03\x88a\x1B\x9BV[\x96P[Pa\x06\xC5V[\x80`\x02\x03a\x05\x11W\x86Qa\x04\x8E\x87`\x02a\x1B\x9BV[\x10a\x04\x9BWa\x04\x9Ba\x1B\xAEV[`\0\x87a\x04\xA9\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x04\xB9Wa\x04\xB9a\x1B\x85V[` \x02` \x01\x01Q\x90P`\x03T\x81\x11\x15a\x05\x06W\x83\x81\x11\x15a\x04\xD9W\x80\x93P[a\x049\x81\x89a\x04\xE9\x8A`\x02a\x1B\x9BV[\x81Q\x81\x10a\x04\xF9Wa\x04\xF9a\x1B\x85V[` \x02` \x01\x01Qa\x0C\xE8V[a\x04p`\x03\x88a\x1B\x9BV[\x80`\x03\x03a\x06\x1BW\x86Qa\x05&\x87`\x04a\x1B\x9BV[\x11\x15a\x054Wa\x054a\x1B\xAEV[`\0\x87a\x05B\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x05RWa\x05Ra\x1B\x85V[` \x02` \x01\x01Q\x90P`\x04T\x81\x11\x15a\x06\x10W\x82\x81\x11\x15a\x05rW\x80\x92P[a\x06\x10\x88a\x05\x81\x89`\x01a\x1B\x9BV[\x81Q\x81\x10a\x05\x91Wa\x05\x91a\x1B\x85V[` \x02` \x01\x01Q\x89\x89`\x02a\x05\xA7\x91\x90a\x1B\x9BV[\x81Q\x81\x10a\x05\xB7Wa\x05\xB7a\x1B\x85V[` \x02` \x01\x01Q\x8A\x8A`\x03a\x05\xCD\x91\x90a\x1B\x9BV[\x81Q\x81\x10a\x05\xDDWa\x05\xDDa\x1B\x85V[` \x02` \x01\x01Q\x8B\x8B`\x04a\x05\xF3\x91\x90a\x1B\x9BV[\x81Q\x81\x10a\x06\x03Wa\x06\x03a\x1B\x85V[` \x02` \x01\x01Qa\x0E\xD2V[a\x04p`\x05\x88a\x1B\x9BV[\x80`\x04\x03a\x06\x85W\x86Qa\x060\x87`\x02a\x1B\x9BV[\x10a\x06=Wa\x06=a\x1B\xAEV[`\0\x87a\x06K\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x06[Wa\x06[a\x1B\x85V[` \x02` \x01\x01Q\x90P`\x03T\x81\x11\x15a\x05\x06W\x83\x81\x11\x15a\x05\x06W\x80\x93Pa\x04p`\x03\x88a\x1B\x9BV[`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x15`$\x82\x01Rt\x12[\x9D\x98[\x1AY\x08\x19\x9A\\\x9C\xDD\x08\x19[\x19[Y[\x9D`Z\x1B`D\x82\x01R`d\x01a\x01\xCFV[Pa\x03\xB2V[`\0\x84g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x06\xE6Wa\x06\xE6a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x07\x0FW\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0[\x85\x81\x10\x15a\x07\\W\x84\x81\x81Q\x81\x10a\x07/Wa\x07/a\x1B\x85V[` \x02` \x01\x01Q\x82\x82\x81Q\x81\x10a\x07IWa\x07Ia\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x07\x15V[P`\x02\x80T\x90`\0a\x07m\x83a\x1B+V[\x90\x91UPP`@\x80Q` \x80\x82\x01\x83R\x83\x82R`\x02T`\0\x90\x81R`\x08\x82R\x92\x90\x92 \x81Q\x80Q\x92\x93\x84\x93a\x07\xA5\x92\x84\x92\x01\x90a\x16\xC5V[P\x90PP\x7F\xE3\xC2\xEBY\x1F\xED\x8A:\xAAo=(\xDD\n\x9D\xB9\xE1\x8B\x80#p\xD3/Q\x01L\xEC\x15\x88\xC5\xD9\xEB\x82`@Qa\x07\xD8\x91\x90a\x1B\xC4V[`@Q\x80\x91\x03\x90\xA1PP`\x04U`\x03UPPPPV[a\x08 `@Q\x80`\xA0\x01`@R\x80``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81RP\x90V[a\x08.`\x03T`\x02Ta\x0F\xF7V[\x90P\x90V[`\x01T`\x01`\x01`\xA0\x1B\x03\x163\x14a\x08}W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\r`$\x82\x01Rl'7\xBA\x10:42\x907\xBB\xB72\xB9`\x99\x1B`D\x82\x01R`d\x01a\x01\xCFV[`\x01a\x08\x89\x82\x80a\x1B\xD7V[\x90P\x11a\x08\xA8W`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x01\xCF\x90a\x1BDV[`\0\x80\x80a\x08\xB6\x84\x80a\x1B\xD7V[\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x08\xD0Wa\x08\xD0a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x08\xF9W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P`\x03T`\x04T\x91\x92P\x90`\0[a\t\x11\x87\x80a\x1B\xD7V[\x90P\x81\x10\x15a\x0B:W6a\t%\x88\x80a\x1B\xD7V[\x83\x81\x81\x10a\t5Wa\t5a\x1B\x85V[\x90P``\x02\x01\x90P`\x03T\x81` \x01` \x81\x01\x90a\tS\x91\x90a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x11a\tgWPa\x0B2V[\x83a\tx`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x11\x15a\t\xA3Wa\t\x97`@\x82\x01` \x83\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x93P[`\0a\t\xB2` \x83\x01\x83a\x1CPV[`\x02\x81\x11\x15a\t\xC3Wa\t\xC3a\x181V[\x03a\n\x10Wa\t\xD8`@\x82\x01` \x83\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x85\x87\x81Q\x81\x10a\t\xF3Wa\t\xF3a\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R\x85a\n\x08\x81a\x1B+V[\x96PPa\x0B0V[`\x01a\n\x1F` \x83\x01\x83a\x1CPV[`\x02\x81\x11\x15a\n0Wa\n0a\x181V[\x03a\n\xCCW`\x03Ta\nH`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x11\x15a\n\xC7Wa\ng``\x82\x01`@\x83\x01a\x1C\x82V[\x15a\n\x94Wa\n\x8Fa\n\x7F`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16`\x01a\x0C\xE8V[a\n\xB7V[a\n\xB7a\n\xA7`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16`\0a\x0C\xE8V[a\t\xD8`@\x82\x01` \x83\x01a\x1C'V[a\x0B0V[`\x02a\n\xDB` \x83\x01\x83a\x1CPV[`\x02\x81\x11\x15a\n\xECWa\n\xECa\x181V[\x14a\x0B0W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x14`$\x82\x01Rsunknown request type``\x1B`D\x82\x01R`d\x01a\x01\xCFV[P[`\x01\x01a\t\x07V[P`\0\x84g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x0BVWa\x0BVa\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x0B\x7FW\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0[\x85\x81\x10\x15a\x07\\W\x84\x81\x81Q\x81\x10a\x0B\x9FWa\x0B\x9Fa\x1B\x85V[` \x02` \x01\x01Q\x82\x82\x81Q\x81\x10a\x0B\xB9Wa\x0B\xB9a\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x0B\x85V[`\x01`\x01`\xA0\x1B\x03\x82\x16a\x0C\x1AW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x15`$\x82\x01RtInvalid token address`X\x1B`D\x82\x01R`d\x01a\x01\xCFV[`\0\x81\x11a\x0CjW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01\x81\x90R`$\x82\x01R\x7FAmount must be greater than zero`D\x82\x01R`d\x01a\x01\xCFV[`\x02\x80T3\x91`\0a\x0C{\x83a\x1B+V[\x90\x91UPP`@\x80Q``\x81\x01\x82R`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x81R\x93\x82\x16` \x80\x86\x01\x91\x82R\x85\x83\x01\x94\x85R`\x02\x80T`\0\x90\x81R`\x07\x90\x92R\x92\x90 \x94Q\x85T\x90\x84\x16`\x01`\x01`\xA0\x1B\x03\x19\x91\x82\x16\x17\x86U\x90Q`\x01\x86\x01\x80T\x91\x90\x94\x16\x91\x16\x17\x90\x91U\x90Q\x91\x01UV[\x80`\x01\x03a\x01eW`\0\x82\x81R`\x07` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81T`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x82R`\x01\x83\x01T\x16\x92\x81\x01\x83\x90R`\x02\x90\x91\x01T\x81\x84\x01\x81\x90R\x92Qcp\xA0\x821`\xE0\x1B\x81R0`\x04\x82\x01R\x90\x92\x90\x82\x90cp\xA0\x821\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\rnW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\r\x92\x91\x90a\x1C\x9FV[\x10\x15a\r\xECW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`$\x80\x82\x01R\x7FInsufficient balance in the cont`D\x82\x01Rc\x1C\x98X\xDD`\xE2\x1B`d\x82\x01R`\x84\x01a\x01\xCFV[\x81Q`@\x80\x84\x01Q\x90Qc\xA9\x05\x9C\xBB`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x16\x92c\xA9\x05\x9C\xBB\x92a\x0E1\x92`\x04\x01`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x82R` \x82\x01R`@\x01\x90V[` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x0EPW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0Et\x91\x90a\x1C\xB8V[P\x81Q` \x80\x84\x01Q`@\x80\x86\x01Q\x81Q`\x01`\x01`\xA0\x1B\x03\x95\x86\x16\x81R\x94\x90\x92\x16\x92\x84\x01\x92\x90\x92R\x82\x82\x01RQ\x7F\xA9/\xF9\x19\xB8P\xE4\x90\x9A\xB2&\x1D\x90~\xF9U\xF1\x1B\xC1qg3\xA6\xCB\xEC\xE3\x8D\x16:i\xAF\x8A\x91\x81\x90\x03``\x01\x90\xA1PPPPV[`\x02\x80T\x90`\0a\x0E\xE2\x83a\x1B+V[\x91\x90PUP`\0`@Q\x80`@\x01`@R\x80`\x05\x81R` \x01dhello`\xD8\x1B\x81RP\x90P`\0\x81`@Q` \x01a\x0F\x1C\x91\x90a\x1C\xD5V[`@\x80Q\x80\x83\x03`\x1F\x19\x01\x81R\x82\x82R\x80Q` \x91\x82\x01 \x83\x83\x01\x83R\x89\x84R\x86\x81\x14\x15\x82\x85\x01\x81\x81R`\x02T`\0\x90\x81R`\x05\x85R\x85\x90 \x86Q\x81U\x90Q`\x01\x90\x91\x01\x80T`\xFF\x19\x16\x91\x15\x15\x91\x90\x91\x17\x90U\x83Q\x88\x81R\x92\x83\x01\x82\x90R\x90\x94P\x92\x91\x7F~\xD2\xB1\x1B\xB9\x8B\xFB\xA3@\xE6\x05\xF2u\xF4\xB4[\xD2O_\xC2:\xC3\xCD\x7F\x9Fa\0\xCD\x1B-{\x1A\x91\x01`@Q\x80\x91\x03\x90\xA1`\x02T`@\x80Q\x91\x82R` \x82\x01\x8A\x90R\x83\x15\x15\x82\x82\x01RQ\x7Ff\xAD1\xC3\x1F\xBC\xF7m\xECf\xF2\x0E\xA9\x04J\xAC\x05'\xE8l\xC1*\xC4\xC3\xEA\xFD\xD0!\x87\xED\t\x91\x91\x81\x90\x03``\x01\x90\xA1PPPPPPPPV[a\x10)`@Q\x80`\xA0\x01`@R\x80``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81RP\x90V[a\x10[`@Q\x80`\xA0\x01`@R\x80``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81RP\x90V[`\0\x80\x80\x80\x80a\x10l\x89`\x01a\x1B\x9BV[\x90P[\x87\x81\x11a\x11.W`\0\x81\x81R`\x06` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x10\xA5W\x84a\x10\x9D\x81a\x1B+V[\x95PPa\x11\x1CV[`\0\x81\x81R`\x07` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x10\xD4W\x83a\x10\xCC\x81a\x1B+V[\x94PPa\x11\x1CV[`\0\x81\x81R`\x08` R`@\x90 T\x15a\x10\xFAW\x81a\x10\xF2\x81a\x1B+V[\x92PPa\x11\x1CV[`\0\x81\x81R`\x05` R`@\x90 T\x15a\x11\x1CW\x82a\x11\x18\x81a\x1B+V[\x93PP[\x80a\x11&\x81a\x1B+V[\x91PPa\x10oV[P`\0\x81\x83a\x11=\x86\x88a\x1B\x9BV[a\x11G\x91\x90a\x1B\x9BV[a\x11Q\x91\x90a\x1B\x9BV[\x90P\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x11lWa\x11la\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x11\x95W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x86R\x83g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x11\xB1Wa\x11\xB1a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x11\xFCW\x81` \x01[`@\x80Q``\x81\x01\x82R`\0\x80\x82R` \x80\x83\x01\x82\x90R\x92\x82\x01R\x82R`\0\x19\x90\x92\x01\x91\x01\x81a\x11\xCFW\x90P[P` \x87\x01R\x84g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x1BWa\x12\x1Ba\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x12fW\x81` \x01[`@\x80Q``\x81\x01\x82R`\0\x80\x82R` \x80\x83\x01\x82\x90R\x92\x82\x01R\x82R`\0\x19\x90\x92\x01\x91\x01\x81a\x129W\x90P[P`@\x87\x01R\x82g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x85Wa\x12\x85a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x12\xCAW\x81` \x01[`@\x80Q\x80\x82\x01\x90\x91R`\0\x80\x82R` \x82\x01R\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x12\xA3W\x90P[P``\x87\x01R\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\xE9Wa\x12\xE9a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x13)W\x81` \x01[`@\x80Q` \x81\x01\x90\x91R``\x81R\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x13\x07W\x90P[P`\x80\x87\x01RP`\0\x93P\x83\x92P\x82\x91P\x81\x90P\x80\x80a\x13J\x8A`\x01a\x1B\x9BV[\x90P[\x88\x81\x11a\x16\xB5W`\0\x81\x81R`\x06` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x14/W\x86Q`\0\x90\x83a\x13\x80\x81a\x1B+V[\x94P\x81Q\x81\x10a\x13\x92Wa\x13\x92a\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x13\xABWa\x13\xABa\x181V[\x90\x81`\x03\x81\x11\x15a\x13\xBEWa\x13\xBEa\x181V[\x90RP`\0\x81\x81R`\x06` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81T`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x82R`\x01\x83\x01T\x16\x92\x81\x01\x92\x90\x92R`\x02\x01T\x81\x83\x01R\x90\x88\x01Q\x87a\x14\r\x81a\x1B+V[\x98P\x81Q\x81\x10a\x14\x1FWa\x14\x1Fa\x1B\x85V[` \x02` \x01\x01\x81\x90RPa\x16\xA3V[`\0\x81\x81R`\x07` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x14\xFBW\x86Q`\x01\x90\x83a\x14[\x81a\x1B+V[\x94P\x81Q\x81\x10a\x14mWa\x14ma\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x14\x86Wa\x14\x86a\x181V[\x90\x81`\x03\x81\x11\x15a\x14\x99Wa\x14\x99a\x181V[\x90RP`\0\x81\x81R`\x07` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81T`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x82R`\x01\x83\x01T\x16\x81\x84\x01R`\x02\x90\x91\x01T\x92\x81\x01\x92\x90\x92R\x88\x01Q\x86a\x14\xE9\x81a\x1B+V[\x97P\x81Q\x81\x10a\x14\x1FWa\x14\x1Fa\x1B\x85V[`\0\x83\x81R`\x08` R`@\x90 T\x15a\x15\xE7W\x86Q`\x03\x90\x83a\x15\x1E\x81a\x1B+V[\x94P\x81Q\x81\x10a\x150Wa\x150a\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x15IWa\x15Ia\x181V[\x90\x81`\x03\x81\x11\x15a\x15\\Wa\x15\\a\x181V[\x90RP`\0\x81\x81R`\x08` \x90\x81R`@\x91\x82\x90 \x82Q\x81T\x80\x84\x02\x82\x01\x85\x01\x85R\x92\x81\x01\x83\x81R\x90\x93\x91\x92\x84\x92\x84\x91\x90\x84\x01\x82\x82\x80\x15a\x15\xBCW` \x02\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R` \x01\x90`\x01\x01\x90\x80\x83\x11a\x15\xA8W[PPPPP\x81RPP\x87`\x80\x01Q\x84\x80a\x15\xD5\x90a\x1B+V[\x95P\x81Q\x81\x10a\x14\x1FWa\x14\x1Fa\x1B\x85V[`\0\x81\x81R`\x05` R`@\x90 T\x15a\x16\xA3W\x86Q`\x02\x90\x83a\x16\n\x81a\x1B+V[\x94P\x81Q\x81\x10a\x16\x1CWa\x16\x1Ca\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x165Wa\x165a\x181V[\x90\x81`\x03\x81\x11\x15a\x16HWa\x16Ha\x181V[\x90RP`\0\x81\x81R`\x05` \x90\x81R`@\x91\x82\x90 \x82Q\x80\x84\x01\x90\x93R\x80T\x83R`\x01\x01T`\xFF\x16\x15\x15\x90\x82\x01R``\x88\x01Q\x85a\x16\x85\x81a\x1B+V[\x96P\x81Q\x81\x10a\x16\x97Wa\x16\x97a\x1B\x85V[` \x02` \x01\x01\x81\x90RP[\x80a\x16\xAD\x81a\x1B+V[\x91PPa\x13MV[P\x94\x95PPPPPP[\x92\x91PPV[\x82\x80T\x82\x82U\x90`\0R` `\0 \x90\x81\x01\x92\x82\x15a\x17\0W\x91` \x02\x82\x01[\x82\x81\x11\x15a\x17\0W\x82Q\x82U\x91` \x01\x91\x90`\x01\x01\x90a\x16\xE5V[Pa\x17\x0C\x92\x91Pa\x17\x10V[P\x90V[[\x80\x82\x11\x15a\x17\x0CW`\0\x81U`\x01\x01a\x17\x11V[`\0\x80`@\x83\x85\x03\x12\x15a\x178W`\0\x80\xFD[\x825`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x17OW`\0\x80\xFD[\x94` \x93\x90\x93\x015\x93PPPV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`\0` \x80\x83\x85\x03\x12\x15a\x17\x86W`\0\x80\xFD[\x825g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a\x17\x9EW`\0\x80\xFD[\x81\x85\x01\x91P\x85`\x1F\x83\x01\x12a\x17\xB2W`\0\x80\xFD[\x815\x81\x81\x11\x15a\x17\xC4Wa\x17\xC4a\x17]V[\x80`\x05\x1B`@Q`\x1F\x19`?\x83\x01\x16\x81\x01\x81\x81\x10\x85\x82\x11\x17\x15a\x17\xE9Wa\x17\xE9a\x17]V[`@R\x91\x82R\x84\x82\x01\x92P\x83\x81\x01\x85\x01\x91\x88\x83\x11\x15a\x18\x07W`\0\x80\xFD[\x93\x85\x01\x93[\x82\x85\x10\x15a\x18%W\x845\x84R\x93\x85\x01\x93\x92\x85\x01\x92a\x18\x0CV[\x98\x97PPPPPPPPV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4Wa\x18\x91\x87\x83Q\x80Q`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x83R` \x80\x83\x01Q\x90\x91\x16\x90\x83\x01R`@\x90\x81\x01Q\x91\x01RV[``\x96\x90\x96\x01\x95\x90\x82\x01\x90`\x01\x01a\x18\\V[P\x94\x95\x94PPPPPV[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4Wa\x18\xF9\x87\x83Q\x80Q`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x83R` \x80\x83\x01Q\x90\x91\x16\x90\x83\x01R`@\x90\x81\x01Q\x91\x01RV[``\x96\x90\x96\x01\x95\x90\x82\x01\x90`\x01\x01a\x18\xC4V[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4W\x81Q\x80Q\x88R\x83\x01Q\x15\x15\x83\x88\x01R`@\x90\x96\x01\x95\x90\x82\x01\x90`\x01\x01a\x19!V[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4W\x81Q\x87R\x95\x82\x01\x95\x90\x82\x01\x90`\x01\x01a\x19_V[`\0\x82\x82Q\x80\x85R` \x80\x86\x01\x95P\x80\x82`\x05\x1B\x84\x01\x01\x81\x86\x01`\0[\x84\x81\x10\x15a\x19\xCEW\x85\x83\x03`\x1F\x19\x01\x89R\x81QQ\x84\x84Ra\x19\xBB\x85\x85\x01\x82a\x19JV[\x99\x85\x01\x99\x93PP\x90\x83\x01\x90`\x01\x01a\x19\x98V[P\x90\x97\x96PPPPPPPV[` \x80\x82R\x82Q`\xA0\x83\x83\x01R\x80Q`\xC0\x84\x01\x81\x90R`\0\x92\x91\x82\x01\x90\x83\x90`\xE0\x86\x01\x90\x82[\x81\x81\x10\x15a\x1A:W\x84Q`\x04\x80\x82\x10a\x1A'WcNH{q`\xE0\x1B\x86R`!\x81R`$\x86\xFD[P\x83R\x93\x85\x01\x93\x91\x85\x01\x91`\x01\x01a\x1A\x01V[PP\x83\x87\x01Q\x93P`\x1F\x19\x92P\x82\x86\x82\x03\x01`@\x87\x01Ra\x1A[\x81\x85a\x18GV[\x93PPP`@\x85\x01Q\x81\x85\x84\x03\x01``\x86\x01Ra\x1Ax\x83\x82a\x18\xAFV[\x92PP``\x85\x01Q\x81\x85\x84\x03\x01`\x80\x86\x01Ra\x1A\x94\x83\x82a\x19\x0CV[\x92PP`\x80\x85\x01Q\x81\x85\x84\x03\x01`\xA0\x86\x01Ra\x1A\xB0\x83\x82a\x19{V[\x96\x95PPPPPPV[`\0` \x82\x84\x03\x12\x15a\x1A\xCCW`\0\x80\xFD[\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x1A\xE3W`\0\x80\xFD[\x82\x01`@\x81\x85\x03\x12\x15a\x1A\xF5W`\0\x80\xFD[\x93\x92PPPV[`\0` \x82\x84\x03\x12\x15a\x1B\x0EW`\0\x80\xFD[P5\x91\x90PV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0`\x01\x82\x01a\x1B=Wa\x1B=a\x1B\x15V[P`\x01\x01\x90V[` \x80\x82R`!\x90\x82\x01R\x7FArray must have at least 1 updat`@\x82\x01R`e`\xF8\x1B``\x82\x01R`\x80\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x80\x82\x01\x80\x82\x11\x15a\x16\xBFWa\x16\xBFa\x1B\x15V[cNH{q`\xE0\x1B`\0R`\x01`\x04R`$`\0\xFD[` \x81R`\0a\x1A\xF5` \x83\x01\x84a\x19JV[`\0\x80\x835`\x1E\x19\x846\x03\x01\x81\x12a\x1B\xEEW`\0\x80\xFD[\x83\x01\x805\x91Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a\x1C\tW`\0\x80\xFD[` \x01\x91P``\x81\x026\x03\x82\x13\x15a\x1C W`\0\x80\xFD[\x92P\x92\x90PV[`\0` \x82\x84\x03\x12\x15a\x1C9W`\0\x80\xFD[\x815`\x01`\x01`\x80\x1B\x03\x81\x16\x81\x14a\x1A\xF5W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a\x1CbW`\0\x80\xFD[\x815`\x03\x81\x10a\x1A\xF5W`\0\x80\xFD[\x80\x15\x15\x81\x14a\x1C\x7FW`\0\x80\xFD[PV[`\0` \x82\x84\x03\x12\x15a\x1C\x94W`\0\x80\xFD[\x815a\x1A\xF5\x81a\x1CqV[`\0` \x82\x84\x03\x12\x15a\x1C\xB1W`\0\x80\xFD[PQ\x91\x90PV[`\0` \x82\x84\x03\x12\x15a\x1C\xCAW`\0\x80\xFD[\x81Qa\x1A\xF5\x81a\x1CqV[`\0\x82Q`\0[\x81\x81\x10\x15a\x1C\xF6W` \x81\x86\x01\x81\x01Q\x85\x83\x01R\x01a\x1C\xDCV[P`\0\x92\x01\x91\x82RP\x91\x90PV\xFE\xA2dipfsX\"\x12 \x11\xF1\x94\x98\xEF\x1B\x94\xDB\xB0{0\xFE\x14I\xD7'|\xFF\xB6\xA0\x18\xFC\x9C\xBA\xF2\xD2o\xF9X\xBBq\ndsolcC\0\x08\x16\x003";
    /// The bytecode of the contract.
    pub static ROLLDOWN_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\0\x9EW`\x005`\xE0\x1C\x80c\xB9\xCF\x81\xBE\x11a\0fW\x80c\xB9\xCF\x81\xBE\x14a\x01\x05W\x80c\xCA\x9B!\xAE\x14a\x01\x18W\x80c\xE1\xB8\xA05\x14a\x01WW\x80c\xF2n\xE9\xD0\x14a\x01iW\x80c\xF3\xFE\xF3\xA3\x14a\x01rW`\0\x80\xFD[\x80cG\xE7\xEF$\x14a\0\xA3W\x80ca\xBC\"\x1A\x14a\0\xB8W\x80c\x7F\xD4\xF8E\x14a\0\xD4W\x80c\x94a<\x16\x14a\0\xDDW\x80c\xB1S\x87\x06\x14a\0\xF0W[`\0\x80\xFD[a\0\xB6a\0\xB16`\x04a\x17%V[a\x01\x85V[\0[a\0\xC1`\x02T\x81V[`@Q\x90\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\0\xC1`\x03T\x81V[a\0\xB6a\0\xEB6`\x04a\x17sV[a\x02\xF3V[a\0\xF8a\x07\xEEV[`@Qa\0\xCB\x91\x90a\x19\xDBV[a\0\xB6a\x01\x136`\x04a\x1A\xBAV[a\x083V[a\x01Ba\x01&6`\x04a\x1A\xFCV[`\x05` R`\0\x90\x81R`@\x90 \x80T`\x01\x90\x91\x01T`\xFF\x16\x82V[`@\x80Q\x92\x83R\x90\x15\x15` \x83\x01R\x01a\0\xCBV[a\0\xB6a\x01e6`\x04a\x17%V[PPV[a\0\xC1`\x04T\x81V[a\0\xB6a\x01\x806`\x04a\x17%V[a\x0B\xCCV[`\x01`\x01`\xA0\x1B\x03\x82\x16a\x01\xD8W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x15`$\x82\x01RtInvalid token address`X\x1B`D\x82\x01R`d\x01[`@Q\x80\x91\x03\x90\xFD[`\0\x81\x11a\x02(W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01\x81\x90R`$\x82\x01R\x7FAmount must be greater than zero`D\x82\x01R`d\x01a\x01\xCFV[`\x02\x80T3\x91\x84\x91\x90`\0a\x02<\x83a\x1B+V[\x90\x91UPP`@\x80Q``\x80\x82\x01\x83R`\x01`\x01`\xA0\x1B\x03\x85\x81\x16\x80\x84R\x88\x82\x16` \x80\x86\x01\x82\x81R\x86\x88\x01\x8B\x81R`\x02\x80T`\0\x90\x81R`\x06\x85R\x8A\x90 \x89Q\x81T\x90\x89\x16`\x01`\x01`\xA0\x1B\x03\x19\x91\x82\x16\x17\x82U\x93Q`\x01\x82\x01\x80T\x91\x90\x99\x16\x94\x16\x93\x90\x93\x17\x90\x96UQ\x94\x01\x93\x90\x93U\x85Q\x91\x82R\x91\x81\x01\x91\x90\x91R\x92\x83\x01\x86\x90R\x90\x91\x7FT\xEED\x88\xADmc\"\x92\x93\xC6\x1F~h\x96\x8C\xD5%\xCE\x18\x8B\xEC\x1EMs\\\x90\x89\x95\xEEP2\x91\x01`@Q\x80\x91\x03\x90\xA1PPPPPV[`\x01T`\x01`\x01`\xA0\x1B\x03\x163\x14a\x03=W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\r`$\x82\x01Rl'7\xBA\x10:42\x907\xBB\xB72\xB9`\x99\x1B`D\x82\x01R`d\x01a\x01\xCFV[`\x02\x81Q\x11a\x03^W`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x01\xCF\x90a\x1BDV[`\0\x80`\0\x83Qg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03}Wa\x03}a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x03\xA6W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P`\x03T`\x04T\x91\x92P\x90[\x85Q\x85\x10\x15a\x06\xCBW`\0\x86\x86\x81Q\x81\x10a\x03\xCFWa\x03\xCFa\x1B\x85V[` \x02` \x01\x01Q\x90P\x80`\x01\x03a\x04yW\x86Qa\x03\xEE\x87`\x02a\x1B\x9BV[\x10a\x03\xFBWa\x03\xFBa\x1B\xAEV[`\0\x87a\x04\t\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x04\x19Wa\x04\x19a\x1B\x85V[` \x02` \x01\x01Q\x90P`\x03T\x81\x11\x15a\x04sW\x83\x81\x11\x15a\x049W\x80\x93P[\x80\x85\x87\x81Q\x81\x10a\x04LWa\x04La\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R\x85a\x04a\x81a\x1B+V[\x96Pa\x04p\x90P`\x03\x88a\x1B\x9BV[\x96P[Pa\x06\xC5V[\x80`\x02\x03a\x05\x11W\x86Qa\x04\x8E\x87`\x02a\x1B\x9BV[\x10a\x04\x9BWa\x04\x9Ba\x1B\xAEV[`\0\x87a\x04\xA9\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x04\xB9Wa\x04\xB9a\x1B\x85V[` \x02` \x01\x01Q\x90P`\x03T\x81\x11\x15a\x05\x06W\x83\x81\x11\x15a\x04\xD9W\x80\x93P[a\x049\x81\x89a\x04\xE9\x8A`\x02a\x1B\x9BV[\x81Q\x81\x10a\x04\xF9Wa\x04\xF9a\x1B\x85V[` \x02` \x01\x01Qa\x0C\xE8V[a\x04p`\x03\x88a\x1B\x9BV[\x80`\x03\x03a\x06\x1BW\x86Qa\x05&\x87`\x04a\x1B\x9BV[\x11\x15a\x054Wa\x054a\x1B\xAEV[`\0\x87a\x05B\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x05RWa\x05Ra\x1B\x85V[` \x02` \x01\x01Q\x90P`\x04T\x81\x11\x15a\x06\x10W\x82\x81\x11\x15a\x05rW\x80\x92P[a\x06\x10\x88a\x05\x81\x89`\x01a\x1B\x9BV[\x81Q\x81\x10a\x05\x91Wa\x05\x91a\x1B\x85V[` \x02` \x01\x01Q\x89\x89`\x02a\x05\xA7\x91\x90a\x1B\x9BV[\x81Q\x81\x10a\x05\xB7Wa\x05\xB7a\x1B\x85V[` \x02` \x01\x01Q\x8A\x8A`\x03a\x05\xCD\x91\x90a\x1B\x9BV[\x81Q\x81\x10a\x05\xDDWa\x05\xDDa\x1B\x85V[` \x02` \x01\x01Q\x8B\x8B`\x04a\x05\xF3\x91\x90a\x1B\x9BV[\x81Q\x81\x10a\x06\x03Wa\x06\x03a\x1B\x85V[` \x02` \x01\x01Qa\x0E\xD2V[a\x04p`\x05\x88a\x1B\x9BV[\x80`\x04\x03a\x06\x85W\x86Qa\x060\x87`\x02a\x1B\x9BV[\x10a\x06=Wa\x06=a\x1B\xAEV[`\0\x87a\x06K\x88`\x01a\x1B\x9BV[\x81Q\x81\x10a\x06[Wa\x06[a\x1B\x85V[` \x02` \x01\x01Q\x90P`\x03T\x81\x11\x15a\x05\x06W\x83\x81\x11\x15a\x05\x06W\x80\x93Pa\x04p`\x03\x88a\x1B\x9BV[`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x15`$\x82\x01Rt\x12[\x9D\x98[\x1AY\x08\x19\x9A\\\x9C\xDD\x08\x19[\x19[Y[\x9D`Z\x1B`D\x82\x01R`d\x01a\x01\xCFV[Pa\x03\xB2V[`\0\x84g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x06\xE6Wa\x06\xE6a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x07\x0FW\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0[\x85\x81\x10\x15a\x07\\W\x84\x81\x81Q\x81\x10a\x07/Wa\x07/a\x1B\x85V[` \x02` \x01\x01Q\x82\x82\x81Q\x81\x10a\x07IWa\x07Ia\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x07\x15V[P`\x02\x80T\x90`\0a\x07m\x83a\x1B+V[\x90\x91UPP`@\x80Q` \x80\x82\x01\x83R\x83\x82R`\x02T`\0\x90\x81R`\x08\x82R\x92\x90\x92 \x81Q\x80Q\x92\x93\x84\x93a\x07\xA5\x92\x84\x92\x01\x90a\x16\xC5V[P\x90PP\x7F\xE3\xC2\xEBY\x1F\xED\x8A:\xAAo=(\xDD\n\x9D\xB9\xE1\x8B\x80#p\xD3/Q\x01L\xEC\x15\x88\xC5\xD9\xEB\x82`@Qa\x07\xD8\x91\x90a\x1B\xC4V[`@Q\x80\x91\x03\x90\xA1PP`\x04U`\x03UPPPPV[a\x08 `@Q\x80`\xA0\x01`@R\x80``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81RP\x90V[a\x08.`\x03T`\x02Ta\x0F\xF7V[\x90P\x90V[`\x01T`\x01`\x01`\xA0\x1B\x03\x163\x14a\x08}W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\r`$\x82\x01Rl'7\xBA\x10:42\x907\xBB\xB72\xB9`\x99\x1B`D\x82\x01R`d\x01a\x01\xCFV[`\x01a\x08\x89\x82\x80a\x1B\xD7V[\x90P\x11a\x08\xA8W`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x01\xCF\x90a\x1BDV[`\0\x80\x80a\x08\xB6\x84\x80a\x1B\xD7V[\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x08\xD0Wa\x08\xD0a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x08\xF9W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P`\x03T`\x04T\x91\x92P\x90`\0[a\t\x11\x87\x80a\x1B\xD7V[\x90P\x81\x10\x15a\x0B:W6a\t%\x88\x80a\x1B\xD7V[\x83\x81\x81\x10a\t5Wa\t5a\x1B\x85V[\x90P``\x02\x01\x90P`\x03T\x81` \x01` \x81\x01\x90a\tS\x91\x90a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x11a\tgWPa\x0B2V[\x83a\tx`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x11\x15a\t\xA3Wa\t\x97`@\x82\x01` \x83\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x93P[`\0a\t\xB2` \x83\x01\x83a\x1CPV[`\x02\x81\x11\x15a\t\xC3Wa\t\xC3a\x181V[\x03a\n\x10Wa\t\xD8`@\x82\x01` \x83\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x85\x87\x81Q\x81\x10a\t\xF3Wa\t\xF3a\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R\x85a\n\x08\x81a\x1B+V[\x96PPa\x0B0V[`\x01a\n\x1F` \x83\x01\x83a\x1CPV[`\x02\x81\x11\x15a\n0Wa\n0a\x181V[\x03a\n\xCCW`\x03Ta\nH`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16\x11\x15a\n\xC7Wa\ng``\x82\x01`@\x83\x01a\x1C\x82V[\x15a\n\x94Wa\n\x8Fa\n\x7F`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16`\x01a\x0C\xE8V[a\n\xB7V[a\n\xB7a\n\xA7`@\x83\x01` \x84\x01a\x1C'V[`\x01`\x01`\x80\x1B\x03\x16`\0a\x0C\xE8V[a\t\xD8`@\x82\x01` \x83\x01a\x1C'V[a\x0B0V[`\x02a\n\xDB` \x83\x01\x83a\x1CPV[`\x02\x81\x11\x15a\n\xECWa\n\xECa\x181V[\x14a\x0B0W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x14`$\x82\x01Rsunknown request type``\x1B`D\x82\x01R`d\x01a\x01\xCFV[P[`\x01\x01a\t\x07V[P`\0\x84g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x0BVWa\x0BVa\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x0B\x7FW\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0[\x85\x81\x10\x15a\x07\\W\x84\x81\x81Q\x81\x10a\x0B\x9FWa\x0B\x9Fa\x1B\x85V[` \x02` \x01\x01Q\x82\x82\x81Q\x81\x10a\x0B\xB9Wa\x0B\xB9a\x1B\x85V[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x0B\x85V[`\x01`\x01`\xA0\x1B\x03\x82\x16a\x0C\x1AW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x15`$\x82\x01RtInvalid token address`X\x1B`D\x82\x01R`d\x01a\x01\xCFV[`\0\x81\x11a\x0CjW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01\x81\x90R`$\x82\x01R\x7FAmount must be greater than zero`D\x82\x01R`d\x01a\x01\xCFV[`\x02\x80T3\x91`\0a\x0C{\x83a\x1B+V[\x90\x91UPP`@\x80Q``\x81\x01\x82R`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x81R\x93\x82\x16` \x80\x86\x01\x91\x82R\x85\x83\x01\x94\x85R`\x02\x80T`\0\x90\x81R`\x07\x90\x92R\x92\x90 \x94Q\x85T\x90\x84\x16`\x01`\x01`\xA0\x1B\x03\x19\x91\x82\x16\x17\x86U\x90Q`\x01\x86\x01\x80T\x91\x90\x94\x16\x91\x16\x17\x90\x91U\x90Q\x91\x01UV[\x80`\x01\x03a\x01eW`\0\x82\x81R`\x07` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81T`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x82R`\x01\x83\x01T\x16\x92\x81\x01\x83\x90R`\x02\x90\x91\x01T\x81\x84\x01\x81\x90R\x92Qcp\xA0\x821`\xE0\x1B\x81R0`\x04\x82\x01R\x90\x92\x90\x82\x90cp\xA0\x821\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\rnW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\r\x92\x91\x90a\x1C\x9FV[\x10\x15a\r\xECW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`$\x80\x82\x01R\x7FInsufficient balance in the cont`D\x82\x01Rc\x1C\x98X\xDD`\xE2\x1B`d\x82\x01R`\x84\x01a\x01\xCFV[\x81Q`@\x80\x84\x01Q\x90Qc\xA9\x05\x9C\xBB`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x84\x16\x92c\xA9\x05\x9C\xBB\x92a\x0E1\x92`\x04\x01`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x82R` \x82\x01R`@\x01\x90V[` `@Q\x80\x83\x03\x81`\0\x87Z\xF1\x15\x80\x15a\x0EPW=`\0\x80>=`\0\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x0Et\x91\x90a\x1C\xB8V[P\x81Q` \x80\x84\x01Q`@\x80\x86\x01Q\x81Q`\x01`\x01`\xA0\x1B\x03\x95\x86\x16\x81R\x94\x90\x92\x16\x92\x84\x01\x92\x90\x92R\x82\x82\x01RQ\x7F\xA9/\xF9\x19\xB8P\xE4\x90\x9A\xB2&\x1D\x90~\xF9U\xF1\x1B\xC1qg3\xA6\xCB\xEC\xE3\x8D\x16:i\xAF\x8A\x91\x81\x90\x03``\x01\x90\xA1PPPPV[`\x02\x80T\x90`\0a\x0E\xE2\x83a\x1B+V[\x91\x90PUP`\0`@Q\x80`@\x01`@R\x80`\x05\x81R` \x01dhello`\xD8\x1B\x81RP\x90P`\0\x81`@Q` \x01a\x0F\x1C\x91\x90a\x1C\xD5V[`@\x80Q\x80\x83\x03`\x1F\x19\x01\x81R\x82\x82R\x80Q` \x91\x82\x01 \x83\x83\x01\x83R\x89\x84R\x86\x81\x14\x15\x82\x85\x01\x81\x81R`\x02T`\0\x90\x81R`\x05\x85R\x85\x90 \x86Q\x81U\x90Q`\x01\x90\x91\x01\x80T`\xFF\x19\x16\x91\x15\x15\x91\x90\x91\x17\x90U\x83Q\x88\x81R\x92\x83\x01\x82\x90R\x90\x94P\x92\x91\x7F~\xD2\xB1\x1B\xB9\x8B\xFB\xA3@\xE6\x05\xF2u\xF4\xB4[\xD2O_\xC2:\xC3\xCD\x7F\x9Fa\0\xCD\x1B-{\x1A\x91\x01`@Q\x80\x91\x03\x90\xA1`\x02T`@\x80Q\x91\x82R` \x82\x01\x8A\x90R\x83\x15\x15\x82\x82\x01RQ\x7Ff\xAD1\xC3\x1F\xBC\xF7m\xECf\xF2\x0E\xA9\x04J\xAC\x05'\xE8l\xC1*\xC4\xC3\xEA\xFD\xD0!\x87\xED\t\x91\x91\x81\x90\x03``\x01\x90\xA1PPPPPPPPV[a\x10)`@Q\x80`\xA0\x01`@R\x80``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81RP\x90V[a\x10[`@Q\x80`\xA0\x01`@R\x80``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81R` \x01``\x81RP\x90V[`\0\x80\x80\x80\x80a\x10l\x89`\x01a\x1B\x9BV[\x90P[\x87\x81\x11a\x11.W`\0\x81\x81R`\x06` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x10\xA5W\x84a\x10\x9D\x81a\x1B+V[\x95PPa\x11\x1CV[`\0\x81\x81R`\x07` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x10\xD4W\x83a\x10\xCC\x81a\x1B+V[\x94PPa\x11\x1CV[`\0\x81\x81R`\x08` R`@\x90 T\x15a\x10\xFAW\x81a\x10\xF2\x81a\x1B+V[\x92PPa\x11\x1CV[`\0\x81\x81R`\x05` R`@\x90 T\x15a\x11\x1CW\x82a\x11\x18\x81a\x1B+V[\x93PP[\x80a\x11&\x81a\x1B+V[\x91PPa\x10oV[P`\0\x81\x83a\x11=\x86\x88a\x1B\x9BV[a\x11G\x91\x90a\x1B\x9BV[a\x11Q\x91\x90a\x1B\x9BV[\x90P\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x11lWa\x11la\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x11\x95W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x86R\x83g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x11\xB1Wa\x11\xB1a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x11\xFCW\x81` \x01[`@\x80Q``\x81\x01\x82R`\0\x80\x82R` \x80\x83\x01\x82\x90R\x92\x82\x01R\x82R`\0\x19\x90\x92\x01\x91\x01\x81a\x11\xCFW\x90P[P` \x87\x01R\x84g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x1BWa\x12\x1Ba\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x12fW\x81` \x01[`@\x80Q``\x81\x01\x82R`\0\x80\x82R` \x80\x83\x01\x82\x90R\x92\x82\x01R\x82R`\0\x19\x90\x92\x01\x91\x01\x81a\x129W\x90P[P`@\x87\x01R\x82g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x85Wa\x12\x85a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x12\xCAW\x81` \x01[`@\x80Q\x80\x82\x01\x90\x91R`\0\x80\x82R` \x82\x01R\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x12\xA3W\x90P[P``\x87\x01R\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\xE9Wa\x12\xE9a\x17]V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x13)W\x81` \x01[`@\x80Q` \x81\x01\x90\x91R``\x81R\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x13\x07W\x90P[P`\x80\x87\x01RP`\0\x93P\x83\x92P\x82\x91P\x81\x90P\x80\x80a\x13J\x8A`\x01a\x1B\x9BV[\x90P[\x88\x81\x11a\x16\xB5W`\0\x81\x81R`\x06` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x14/W\x86Q`\0\x90\x83a\x13\x80\x81a\x1B+V[\x94P\x81Q\x81\x10a\x13\x92Wa\x13\x92a\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x13\xABWa\x13\xABa\x181V[\x90\x81`\x03\x81\x11\x15a\x13\xBEWa\x13\xBEa\x181V[\x90RP`\0\x81\x81R`\x06` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81T`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x82R`\x01\x83\x01T\x16\x92\x81\x01\x92\x90\x92R`\x02\x01T\x81\x83\x01R\x90\x88\x01Q\x87a\x14\r\x81a\x1B+V[\x98P\x81Q\x81\x10a\x14\x1FWa\x14\x1Fa\x1B\x85V[` \x02` \x01\x01\x81\x90RPa\x16\xA3V[`\0\x81\x81R`\x07` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x15a\x14\xFBW\x86Q`\x01\x90\x83a\x14[\x81a\x1B+V[\x94P\x81Q\x81\x10a\x14mWa\x14ma\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x14\x86Wa\x14\x86a\x181V[\x90\x81`\x03\x81\x11\x15a\x14\x99Wa\x14\x99a\x181V[\x90RP`\0\x81\x81R`\x07` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81T`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x82R`\x01\x83\x01T\x16\x81\x84\x01R`\x02\x90\x91\x01T\x92\x81\x01\x92\x90\x92R\x88\x01Q\x86a\x14\xE9\x81a\x1B+V[\x97P\x81Q\x81\x10a\x14\x1FWa\x14\x1Fa\x1B\x85V[`\0\x83\x81R`\x08` R`@\x90 T\x15a\x15\xE7W\x86Q`\x03\x90\x83a\x15\x1E\x81a\x1B+V[\x94P\x81Q\x81\x10a\x150Wa\x150a\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x15IWa\x15Ia\x181V[\x90\x81`\x03\x81\x11\x15a\x15\\Wa\x15\\a\x181V[\x90RP`\0\x81\x81R`\x08` \x90\x81R`@\x91\x82\x90 \x82Q\x81T\x80\x84\x02\x82\x01\x85\x01\x85R\x92\x81\x01\x83\x81R\x90\x93\x91\x92\x84\x92\x84\x91\x90\x84\x01\x82\x82\x80\x15a\x15\xBCW` \x02\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R` \x01\x90`\x01\x01\x90\x80\x83\x11a\x15\xA8W[PPPPP\x81RPP\x87`\x80\x01Q\x84\x80a\x15\xD5\x90a\x1B+V[\x95P\x81Q\x81\x10a\x14\x1FWa\x14\x1Fa\x1B\x85V[`\0\x81\x81R`\x05` R`@\x90 T\x15a\x16\xA3W\x86Q`\x02\x90\x83a\x16\n\x81a\x1B+V[\x94P\x81Q\x81\x10a\x16\x1CWa\x16\x1Ca\x1B\x85V[` \x02` \x01\x01\x90`\x03\x81\x11\x15a\x165Wa\x165a\x181V[\x90\x81`\x03\x81\x11\x15a\x16HWa\x16Ha\x181V[\x90RP`\0\x81\x81R`\x05` \x90\x81R`@\x91\x82\x90 \x82Q\x80\x84\x01\x90\x93R\x80T\x83R`\x01\x01T`\xFF\x16\x15\x15\x90\x82\x01R``\x88\x01Q\x85a\x16\x85\x81a\x1B+V[\x96P\x81Q\x81\x10a\x16\x97Wa\x16\x97a\x1B\x85V[` \x02` \x01\x01\x81\x90RP[\x80a\x16\xAD\x81a\x1B+V[\x91PPa\x13MV[P\x94\x95PPPPPP[\x92\x91PPV[\x82\x80T\x82\x82U\x90`\0R` `\0 \x90\x81\x01\x92\x82\x15a\x17\0W\x91` \x02\x82\x01[\x82\x81\x11\x15a\x17\0W\x82Q\x82U\x91` \x01\x91\x90`\x01\x01\x90a\x16\xE5V[Pa\x17\x0C\x92\x91Pa\x17\x10V[P\x90V[[\x80\x82\x11\x15a\x17\x0CW`\0\x81U`\x01\x01a\x17\x11V[`\0\x80`@\x83\x85\x03\x12\x15a\x178W`\0\x80\xFD[\x825`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x17OW`\0\x80\xFD[\x94` \x93\x90\x93\x015\x93PPPV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`\0` \x80\x83\x85\x03\x12\x15a\x17\x86W`\0\x80\xFD[\x825g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x80\x82\x11\x15a\x17\x9EW`\0\x80\xFD[\x81\x85\x01\x91P\x85`\x1F\x83\x01\x12a\x17\xB2W`\0\x80\xFD[\x815\x81\x81\x11\x15a\x17\xC4Wa\x17\xC4a\x17]V[\x80`\x05\x1B`@Q`\x1F\x19`?\x83\x01\x16\x81\x01\x81\x81\x10\x85\x82\x11\x17\x15a\x17\xE9Wa\x17\xE9a\x17]V[`@R\x91\x82R\x84\x82\x01\x92P\x83\x81\x01\x85\x01\x91\x88\x83\x11\x15a\x18\x07W`\0\x80\xFD[\x93\x85\x01\x93[\x82\x85\x10\x15a\x18%W\x845\x84R\x93\x85\x01\x93\x92\x85\x01\x92a\x18\x0CV[\x98\x97PPPPPPPPV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4Wa\x18\x91\x87\x83Q\x80Q`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x83R` \x80\x83\x01Q\x90\x91\x16\x90\x83\x01R`@\x90\x81\x01Q\x91\x01RV[``\x96\x90\x96\x01\x95\x90\x82\x01\x90`\x01\x01a\x18\\V[P\x94\x95\x94PPPPPV[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4Wa\x18\xF9\x87\x83Q\x80Q`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x83R` \x80\x83\x01Q\x90\x91\x16\x90\x83\x01R`@\x90\x81\x01Q\x91\x01RV[``\x96\x90\x96\x01\x95\x90\x82\x01\x90`\x01\x01a\x18\xC4V[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4W\x81Q\x80Q\x88R\x83\x01Q\x15\x15\x83\x88\x01R`@\x90\x96\x01\x95\x90\x82\x01\x90`\x01\x01a\x19!V[`\0\x81Q\x80\x84R` \x80\x85\x01\x94P` \x84\x01`\0[\x83\x81\x10\x15a\x18\xA4W\x81Q\x87R\x95\x82\x01\x95\x90\x82\x01\x90`\x01\x01a\x19_V[`\0\x82\x82Q\x80\x85R` \x80\x86\x01\x95P\x80\x82`\x05\x1B\x84\x01\x01\x81\x86\x01`\0[\x84\x81\x10\x15a\x19\xCEW\x85\x83\x03`\x1F\x19\x01\x89R\x81QQ\x84\x84Ra\x19\xBB\x85\x85\x01\x82a\x19JV[\x99\x85\x01\x99\x93PP\x90\x83\x01\x90`\x01\x01a\x19\x98V[P\x90\x97\x96PPPPPPPV[` \x80\x82R\x82Q`\xA0\x83\x83\x01R\x80Q`\xC0\x84\x01\x81\x90R`\0\x92\x91\x82\x01\x90\x83\x90`\xE0\x86\x01\x90\x82[\x81\x81\x10\x15a\x1A:W\x84Q`\x04\x80\x82\x10a\x1A'WcNH{q`\xE0\x1B\x86R`!\x81R`$\x86\xFD[P\x83R\x93\x85\x01\x93\x91\x85\x01\x91`\x01\x01a\x1A\x01V[PP\x83\x87\x01Q\x93P`\x1F\x19\x92P\x82\x86\x82\x03\x01`@\x87\x01Ra\x1A[\x81\x85a\x18GV[\x93PPP`@\x85\x01Q\x81\x85\x84\x03\x01``\x86\x01Ra\x1Ax\x83\x82a\x18\xAFV[\x92PP``\x85\x01Q\x81\x85\x84\x03\x01`\x80\x86\x01Ra\x1A\x94\x83\x82a\x19\x0CV[\x92PP`\x80\x85\x01Q\x81\x85\x84\x03\x01`\xA0\x86\x01Ra\x1A\xB0\x83\x82a\x19{V[\x96\x95PPPPPPV[`\0` \x82\x84\x03\x12\x15a\x1A\xCCW`\0\x80\xFD[\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x1A\xE3W`\0\x80\xFD[\x82\x01`@\x81\x85\x03\x12\x15a\x1A\xF5W`\0\x80\xFD[\x93\x92PPPV[`\0` \x82\x84\x03\x12\x15a\x1B\x0EW`\0\x80\xFD[P5\x91\x90PV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0`\x01\x82\x01a\x1B=Wa\x1B=a\x1B\x15V[P`\x01\x01\x90V[` \x80\x82R`!\x90\x82\x01R\x7FArray must have at least 1 updat`@\x82\x01R`e`\xF8\x1B``\x82\x01R`\x80\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x80\x82\x01\x80\x82\x11\x15a\x16\xBFWa\x16\xBFa\x1B\x15V[cNH{q`\xE0\x1B`\0R`\x01`\x04R`$`\0\xFD[` \x81R`\0a\x1A\xF5` \x83\x01\x84a\x19JV[`\0\x80\x835`\x1E\x19\x846\x03\x01\x81\x12a\x1B\xEEW`\0\x80\xFD[\x83\x01\x805\x91Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x15a\x1C\tW`\0\x80\xFD[` \x01\x91P``\x81\x026\x03\x82\x13\x15a\x1C W`\0\x80\xFD[\x92P\x92\x90PV[`\0` \x82\x84\x03\x12\x15a\x1C9W`\0\x80\xFD[\x815`\x01`\x01`\x80\x1B\x03\x81\x16\x81\x14a\x1A\xF5W`\0\x80\xFD[`\0` \x82\x84\x03\x12\x15a\x1CbW`\0\x80\xFD[\x815`\x03\x81\x10a\x1A\xF5W`\0\x80\xFD[\x80\x15\x15\x81\x14a\x1C\x7FW`\0\x80\xFD[PV[`\0` \x82\x84\x03\x12\x15a\x1C\x94W`\0\x80\xFD[\x815a\x1A\xF5\x81a\x1CqV[`\0` \x82\x84\x03\x12\x15a\x1C\xB1W`\0\x80\xFD[PQ\x91\x90PV[`\0` \x82\x84\x03\x12\x15a\x1C\xCAW`\0\x80\xFD[\x81Qa\x1A\xF5\x81a\x1CqV[`\0\x82Q`\0[\x81\x81\x10\x15a\x1C\xF6W` \x81\x86\x01\x81\x01Q\x85\x83\x01R\x01a\x1C\xDCV[P`\0\x92\x01\x91\x82RP\x91\x90PV\xFE\xA2dipfsX\"\x12 \x11\xF1\x94\x98\xEF\x1B\x94\xDB\xB0{0\xFE\x14I\xD7'|\xFF\xB6\xA0\x18\xFC\x9C\xBA\xF2\xD2o\xF9X\xBBq\ndsolcC\0\x08\x16\x003";
    /// The deployed bytecode of the contract.
    pub static ROLLDOWN_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct RollDown<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for RollDown<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for RollDown<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for RollDown<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for RollDown<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(RollDown)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> RollDown<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    ROLLDOWN_ABI.clone(),
                    client,
                ),
            )
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// - If there are no constructor arguments, you should pass `()` as the argument.
        /// - The default poll duration is 7 seconds.
        /// - The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter, "../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::core::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                ROLLDOWN_ABI.clone(),
                ROLLDOWN_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `cancelResolutions` (0xca9b21ae) function
        pub fn cancel_resolutions(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::ethers::core::types::U256, bool),
        > {
            self.0
                .method_hash([202, 155, 33, 174], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `counter` (0x61bc221a) function
        pub fn counter(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([97, 188, 34, 26], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deposit` (0x47e7ef24) function
        pub fn deposit(
            &self,
            token_address: ::ethers::core::types::Address,
            amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([71, 231, 239, 36], (token_address, amount))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getUpdateForL2` (0xb1538706) function
        pub fn get_update_for_l2(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, L1Update> {
            self.0
                .method_hash([177, 83, 135, 6], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastProcessedUpdate_origin_l1` (0x7fd4f845) function
        pub fn last_processed_update_origin_l_1(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([127, 212, 248, 69], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastProcessedUpdate_origin_l2` (0xf26ee9d0) function
        pub fn last_processed_update_origin_l_2(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([242, 110, 233, 208], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `mat` (0xe1b8a035) function
        pub fn mat(
            &self,
            token_address: ::ethers::core::types::Address,
            token_address_2: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([225, 184, 160, 53], (token_address, token_address_2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `update_l1_from_l2` (0x94613c16) function
        pub fn update_l_1_from_l_2(
            &self,
            input_array: ::std::vec::Vec<::ethers::core::types::U256>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([148, 97, 60, 22], input_array)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `update_l1_from_l2_mat` (0xb9cf81be) function
        pub fn update_l_1_from_l_2_mat(
            &self,
            input_array: L2ToL1Update,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([185, 207, 129, 190], (input_array,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `withdraw` (0xf3fef3a3) function
        pub fn withdraw(
            &self,
            token_address: ::ethers::core::types::Address,
            amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([243, 254, 243, 163], (token_address, amount))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `DepositAcceptedIntoQueue` event
        pub fn deposit_accepted_into_queue_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            DepositAcceptedIntoQueueFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `DisputeResolutionAcceptedIntoQueue` event
        pub fn dispute_resolution_accepted_into_queue_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            DisputeResolutionAcceptedIntoQueueFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `FundsWithdrawn` event
        pub fn funds_withdrawn_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            FundsWithdrawnFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `L2UpdatesToRemovedAcceptedIntoQueue` event
        pub fn l2_updates_to_removed_accepted_into_queue_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            L2UpdatesToRemovedAcceptedIntoQueueFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `WithdrawAcceptedIntoQueue` event
        pub fn withdraw_accepted_into_queue_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            WithdrawAcceptedIntoQueueFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `cancelAndCalculatedHash` event
        pub fn cancel_and_calculated_hash_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CancelAndCalculatedHashFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RollDownEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for RollDown<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "DepositAcceptedIntoQueue",
        abi = "DepositAcceptedIntoQueue(address,address,uint256)"
    )]
    pub struct DepositAcceptedIntoQueueFilter {
        pub deposit_recipient: ::ethers::core::types::Address,
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "DisputeResolutionAcceptedIntoQueue",
        abi = "DisputeResolutionAcceptedIntoQueue(uint256,uint256,bool)"
    )]
    pub struct DisputeResolutionAcceptedIntoQueueFilter {
        pub request_id: ::ethers::core::types::U256,
        pub original_request_id: ::ethers::core::types::U256,
        pub cancel_justified: bool,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "FundsWithdrawn", abi = "FundsWithdrawn(address,address,uint256)")]
    pub struct FundsWithdrawnFilter {
        pub withdraw_recipient: ::ethers::core::types::Address,
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "L2UpdatesToRemovedAcceptedIntoQueue",
        abi = "L2UpdatesToRemovedAcceptedIntoQueue(uint256[])"
    )]
    pub struct L2UpdatesToRemovedAcceptedIntoQueueFilter {
        pub l_2_updates_to_remove: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "WithdrawAcceptedIntoQueue",
        abi = "WithdrawAcceptedIntoQueue(address,address,uint256)"
    )]
    pub struct WithdrawAcceptedIntoQueueFilter {
        pub withdraw_recipient: ::ethers::core::types::Address,
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "cancelAndCalculatedHash",
        abi = "cancelAndCalculatedHash(bytes32,bytes32)"
    )]
    pub struct CancelAndCalculatedHashFilter {
        pub cancel_hash: [u8; 32],
        pub calculated_hash: [u8; 32],
    }
    ///Container type for all of the contract's events
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        serde::Serialize,
        serde::Deserialize,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub enum RollDownEvents {
        DepositAcceptedIntoQueueFilter(DepositAcceptedIntoQueueFilter),
        DisputeResolutionAcceptedIntoQueueFilter(
            DisputeResolutionAcceptedIntoQueueFilter,
        ),
        FundsWithdrawnFilter(FundsWithdrawnFilter),
        L2UpdatesToRemovedAcceptedIntoQueueFilter(
            L2UpdatesToRemovedAcceptedIntoQueueFilter,
        ),
        WithdrawAcceptedIntoQueueFilter(WithdrawAcceptedIntoQueueFilter),
        CancelAndCalculatedHashFilter(CancelAndCalculatedHashFilter),
    }
    impl ::ethers::contract::EthLogDecode for RollDownEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = DepositAcceptedIntoQueueFilter::decode_log(log) {
                return Ok(RollDownEvents::DepositAcceptedIntoQueueFilter(decoded));
            }
            if let Ok(decoded) = DisputeResolutionAcceptedIntoQueueFilter::decode_log(
                log,
            ) {
                return Ok(
                    RollDownEvents::DisputeResolutionAcceptedIntoQueueFilter(decoded),
                );
            }
            if let Ok(decoded) = FundsWithdrawnFilter::decode_log(log) {
                return Ok(RollDownEvents::FundsWithdrawnFilter(decoded));
            }
            if let Ok(decoded) = L2UpdatesToRemovedAcceptedIntoQueueFilter::decode_log(
                log,
            ) {
                return Ok(
                    RollDownEvents::L2UpdatesToRemovedAcceptedIntoQueueFilter(decoded),
                );
            }
            if let Ok(decoded) = WithdrawAcceptedIntoQueueFilter::decode_log(log) {
                return Ok(RollDownEvents::WithdrawAcceptedIntoQueueFilter(decoded));
            }
            if let Ok(decoded) = CancelAndCalculatedHashFilter::decode_log(log) {
                return Ok(RollDownEvents::CancelAndCalculatedHashFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for RollDownEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DepositAcceptedIntoQueueFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DisputeResolutionAcceptedIntoQueueFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FundsWithdrawnFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::L2UpdatesToRemovedAcceptedIntoQueueFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::WithdrawAcceptedIntoQueueFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CancelAndCalculatedHashFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<DepositAcceptedIntoQueueFilter> for RollDownEvents {
        fn from(value: DepositAcceptedIntoQueueFilter) -> Self {
            Self::DepositAcceptedIntoQueueFilter(value)
        }
    }
    impl ::core::convert::From<DisputeResolutionAcceptedIntoQueueFilter>
    for RollDownEvents {
        fn from(value: DisputeResolutionAcceptedIntoQueueFilter) -> Self {
            Self::DisputeResolutionAcceptedIntoQueueFilter(value)
        }
    }
    impl ::core::convert::From<FundsWithdrawnFilter> for RollDownEvents {
        fn from(value: FundsWithdrawnFilter) -> Self {
            Self::FundsWithdrawnFilter(value)
        }
    }
    impl ::core::convert::From<L2UpdatesToRemovedAcceptedIntoQueueFilter>
    for RollDownEvents {
        fn from(value: L2UpdatesToRemovedAcceptedIntoQueueFilter) -> Self {
            Self::L2UpdatesToRemovedAcceptedIntoQueueFilter(value)
        }
    }
    impl ::core::convert::From<WithdrawAcceptedIntoQueueFilter> for RollDownEvents {
        fn from(value: WithdrawAcceptedIntoQueueFilter) -> Self {
            Self::WithdrawAcceptedIntoQueueFilter(value)
        }
    }
    impl ::core::convert::From<CancelAndCalculatedHashFilter> for RollDownEvents {
        fn from(value: CancelAndCalculatedHashFilter) -> Self {
            Self::CancelAndCalculatedHashFilter(value)
        }
    }
    ///Container type for all input parameters for the `cancelResolutions` function with signature `cancelResolutions(uint256)` and selector `0xca9b21ae`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "cancelResolutions", abi = "cancelResolutions(uint256)")]
    pub struct CancelResolutionsCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `counter` function with signature `counter()` and selector `0x61bc221a`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "counter", abi = "counter()")]
    pub struct CounterCall;
    ///Container type for all input parameters for the `deposit` function with signature `deposit(address,uint256)` and selector `0x47e7ef24`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deposit", abi = "deposit(address,uint256)")]
    pub struct DepositCall {
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `getUpdateForL2` function with signature `getUpdateForL2()` and selector `0xb1538706`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getUpdateForL2", abi = "getUpdateForL2()")]
    pub struct GetUpdateForL2Call;
    ///Container type for all input parameters for the `lastProcessedUpdate_origin_l1` function with signature `lastProcessedUpdate_origin_l1()` and selector `0x7fd4f845`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "lastProcessedUpdate_origin_l1",
        abi = "lastProcessedUpdate_origin_l1()"
    )]
    pub struct LastProcessedUpdateOriginL1Call;
    ///Container type for all input parameters for the `lastProcessedUpdate_origin_l2` function with signature `lastProcessedUpdate_origin_l2()` and selector `0xf26ee9d0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "lastProcessedUpdate_origin_l2",
        abi = "lastProcessedUpdate_origin_l2()"
    )]
    pub struct LastProcessedUpdateOriginL2Call;
    ///Container type for all input parameters for the `mat` function with signature `mat(address,uint256)` and selector `0xe1b8a035`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "mat", abi = "mat(address,uint256)")]
    pub struct MatCall {
        pub token_address: ::ethers::core::types::Address,
        pub token_address_2: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `update_l1_from_l2` function with signature `update_l1_from_l2(uint256[])` and selector `0x94613c16`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "update_l1_from_l2", abi = "update_l1_from_l2(uint256[])")]
    pub struct UpdateL1FromL2Call {
        pub input_array: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all input parameters for the `update_l1_from_l2_mat` function with signature `update_l1_from_l2_mat(((uint8,uint128,bool)[],(bytes32,bytes32,uint128,uint128,bytes32)[]))` and selector `0xb9cf81be`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "update_l1_from_l2_mat",
        abi = "update_l1_from_l2_mat(((uint8,uint128,bool)[],(bytes32,bytes32,uint128,uint128,bytes32)[]))"
    )]
    pub struct UpdateL1FromL2MatCall {
        pub input_array: L2ToL1Update,
    }
    ///Container type for all input parameters for the `withdraw` function with signature `withdraw(address,uint256)` and selector `0xf3fef3a3`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "withdraw", abi = "withdraw(address,uint256)")]
    pub struct WithdrawCall {
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's call
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        serde::Serialize,
        serde::Deserialize,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub enum RollDownCalls {
        CancelResolutions(CancelResolutionsCall),
        Counter(CounterCall),
        Deposit(DepositCall),
        GetUpdateForL2(GetUpdateForL2Call),
        LastProcessedUpdateOriginL1(LastProcessedUpdateOriginL1Call),
        LastProcessedUpdateOriginL2(LastProcessedUpdateOriginL2Call),
        Mat(MatCall),
        UpdateL1FromL2(UpdateL1FromL2Call),
        UpdateL1FromL2Mat(UpdateL1FromL2MatCall),
        Withdraw(WithdrawCall),
    }
    impl ::ethers::core::abi::AbiDecode for RollDownCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <CancelResolutionsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CancelResolutions(decoded));
            }
            if let Ok(decoded) = <CounterCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Counter(decoded));
            }
            if let Ok(decoded) = <DepositCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Deposit(decoded));
            }
            if let Ok(decoded) = <GetUpdateForL2Call as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetUpdateForL2(decoded));
            }
            if let Ok(decoded) = <LastProcessedUpdateOriginL1Call as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastProcessedUpdateOriginL1(decoded));
            }
            if let Ok(decoded) = <LastProcessedUpdateOriginL2Call as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastProcessedUpdateOriginL2(decoded));
            }
            if let Ok(decoded) = <MatCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Mat(decoded));
            }
            if let Ok(decoded) = <UpdateL1FromL2Call as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateL1FromL2(decoded));
            }
            if let Ok(decoded) = <UpdateL1FromL2MatCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateL1FromL2Mat(decoded));
            }
            if let Ok(decoded) = <WithdrawCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Withdraw(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for RollDownCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::CancelResolutions(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Counter(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Deposit(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetUpdateForL2(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastProcessedUpdateOriginL1(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastProcessedUpdateOriginL2(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Mat(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::UpdateL1FromL2(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateL1FromL2Mat(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Withdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for RollDownCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CancelResolutions(element) => ::core::fmt::Display::fmt(element, f),
                Self::Counter(element) => ::core::fmt::Display::fmt(element, f),
                Self::Deposit(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetUpdateForL2(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastProcessedUpdateOriginL1(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::LastProcessedUpdateOriginL2(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Mat(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateL1FromL2(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateL1FromL2Mat(element) => ::core::fmt::Display::fmt(element, f),
                Self::Withdraw(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CancelResolutionsCall> for RollDownCalls {
        fn from(value: CancelResolutionsCall) -> Self {
            Self::CancelResolutions(value)
        }
    }
    impl ::core::convert::From<CounterCall> for RollDownCalls {
        fn from(value: CounterCall) -> Self {
            Self::Counter(value)
        }
    }
    impl ::core::convert::From<DepositCall> for RollDownCalls {
        fn from(value: DepositCall) -> Self {
            Self::Deposit(value)
        }
    }
    impl ::core::convert::From<GetUpdateForL2Call> for RollDownCalls {
        fn from(value: GetUpdateForL2Call) -> Self {
            Self::GetUpdateForL2(value)
        }
    }
    impl ::core::convert::From<LastProcessedUpdateOriginL1Call> for RollDownCalls {
        fn from(value: LastProcessedUpdateOriginL1Call) -> Self {
            Self::LastProcessedUpdateOriginL1(value)
        }
    }
    impl ::core::convert::From<LastProcessedUpdateOriginL2Call> for RollDownCalls {
        fn from(value: LastProcessedUpdateOriginL2Call) -> Self {
            Self::LastProcessedUpdateOriginL2(value)
        }
    }
    impl ::core::convert::From<MatCall> for RollDownCalls {
        fn from(value: MatCall) -> Self {
            Self::Mat(value)
        }
    }
    impl ::core::convert::From<UpdateL1FromL2Call> for RollDownCalls {
        fn from(value: UpdateL1FromL2Call) -> Self {
            Self::UpdateL1FromL2(value)
        }
    }
    impl ::core::convert::From<UpdateL1FromL2MatCall> for RollDownCalls {
        fn from(value: UpdateL1FromL2MatCall) -> Self {
            Self::UpdateL1FromL2Mat(value)
        }
    }
    impl ::core::convert::From<WithdrawCall> for RollDownCalls {
        fn from(value: WithdrawCall) -> Self {
            Self::Withdraw(value)
        }
    }
    ///Container type for all return fields from the `cancelResolutions` function with signature `cancelResolutions(uint256)` and selector `0xca9b21ae`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CancelResolutionsReturn {
        pub l_2_request_id: ::ethers::core::types::U256,
        pub cancel_justified: bool,
    }
    ///Container type for all return fields from the `counter` function with signature `counter()` and selector `0x61bc221a`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CounterReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getUpdateForL2` function with signature `getUpdateForL2()` and selector `0xb1538706`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetUpdateForL2Return(pub L1Update);
    ///Container type for all return fields from the `lastProcessedUpdate_origin_l1` function with signature `lastProcessedUpdate_origin_l1()` and selector `0x7fd4f845`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LastProcessedUpdateOriginL1Return(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `lastProcessedUpdate_origin_l2` function with signature `lastProcessedUpdate_origin_l2()` and selector `0xf26ee9d0`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LastProcessedUpdateOriginL2Return(pub ::ethers::core::types::U256);
    ///`CancelEth(bytes32,bytes32,uint128,uint128,bytes32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CancelEth {
        pub updater: [u8; 32],
        pub canceler: [u8; 32],
        pub last_proccessed_request_on_l1: u128,
        pub last_accepted_request_on_l1: u128,
        pub hash: [u8; 32],
    }
    ///`CancelResolution(uint256,bool)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CancelResolution {
        pub l_2_request_id: ::ethers::core::types::U256,
        pub cancel_justified: bool,
    }
    ///`Deposit(address,address,uint256)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Deposit {
        pub deposit_recipient: ::ethers::core::types::Address,
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    ///`L1Update(uint8[],(address,address,uint256)[],(address,address,uint256)[],(uint256,bool)[],(uint256[])[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct L1Update {
        pub order: ::std::vec::Vec<u8>,
        pub pending_withdraws: ::std::vec::Vec<Withdraw>,
        pub pending_deposits: ::std::vec::Vec<Deposit>,
        pub pending_cancel_resultions: ::std::vec::Vec<CancelResolution>,
        pub pending_l2_updates_to_remove: ::std::vec::Vec<L2UpdatesToRemove>,
    }
    ///`L2ToL1Update((uint8,uint128,bool)[],(bytes32,bytes32,uint128,uint128,bytes32)[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct L2ToL1Update {
        pub updates: ::std::vec::Vec<StatusUpdate>,
        pub cancels: ::std::vec::Vec<CancelEth>,
    }
    ///`L2UpdatesToRemove(uint256[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct L2UpdatesToRemove {
        pub l_2_updates_to_remove: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///`StatusUpdate(uint8,uint128,bool)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StatusUpdate {
        pub update_type: u8,
        pub request_id: u128,
        pub status: bool,
    }
    ///`Withdraw(address,address,uint256)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        serde::Serialize,
        serde::Deserialize,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct Withdraw {
        pub withdraw_recipient: ::ethers::core::types::Address,
        pub token_address: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
}
