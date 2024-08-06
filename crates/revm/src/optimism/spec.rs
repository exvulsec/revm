use revm_precompile::PrecompileSpecId;

use crate::{
    handler::register::HandleRegisters,
    primitives::{db::Database, BlockEnv, Spec, SpecId},
    EvmHandler, L1BlockInfo,
};

use super::{env::TxEnv, OptimismContext, OptimismHaltReason};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EvmWiring;

impl crate::primitives::EvmWiring for EvmWiring {
    type Block = BlockEnv;
    type Hardfork = OptimismSpecId;
    type HaltReason = OptimismHaltReason;
    type Transaction = TxEnv;
}

impl crate::EvmWiring for EvmWiring {
    type Context = Context;

    fn handler<'evm, EXT, DB>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self, EXT, DB>
    where
        DB: Database,
    {
        let mut handler = EvmHandler::mainnet_with_spec(hardfork);

        handler.append_handler_register(HandleRegisters::Plain(
            crate::optimism::optimism_handle_register::<Self, DB, EXT>,
        ));

        handler
    }
}

/// Context for the Optimism chain.
#[derive(Default)]
pub struct Context {
    l1_block_info: Option<L1BlockInfo>,
}

impl OptimismContext for Context {
    fn l1_block_info(&self) -> Option<&L1BlockInfo> {
        self.l1_block_info.as_ref()
    }

    fn l1_block_info_mut(&mut self) -> &mut Option<L1BlockInfo> {
        &mut self.l1_block_info
    }
}

/// Specification IDs for the optimism blockchain.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum OptimismSpecId {
    FRONTIER = 0,
    FRONTIER_THAWING = 1,
    HOMESTEAD = 2,
    DAO_FORK = 3,
    TANGERINE = 4,
    SPURIOUS_DRAGON = 5,
    BYZANTIUM = 6,
    CONSTANTINOPLE = 7,
    PETERSBURG = 8,
    ISTANBUL = 9,
    MUIR_GLACIER = 10,
    BERLIN = 11,
    LONDON = 12,
    ARROW_GLACIER = 13,
    GRAY_GLACIER = 14,
    MERGE = 15,
    BEDROCK = 16,
    REGOLITH = 17,
    SHANGHAI = 18,
    CANYON = 19,
    CANCUN = 20,
    ECOTONE = 21,
    FJORD = 22,
    PRAGUE = 23,
    PRAGUE_EOF = 24,
    #[default]
    LATEST = u8::MAX,
}

impl OptimismSpecId {
    /// Returns the `OptimismSpecId` for the given `u8`.
    #[inline]
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::n(spec_id)
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        Self::enabled(self, other)
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn enabled(our: Self, other: Self) -> bool {
        our as u8 >= other as u8
    }

    /// Converts the `OptimismSpecId` into a `SpecId`.
    const fn into_eth_spec_id(self) -> SpecId {
        match self {
            OptimismSpecId::FRONTIER => SpecId::FRONTIER,
            OptimismSpecId::FRONTIER_THAWING => SpecId::FRONTIER_THAWING,
            OptimismSpecId::HOMESTEAD => SpecId::HOMESTEAD,
            OptimismSpecId::DAO_FORK => SpecId::DAO_FORK,
            OptimismSpecId::TANGERINE => SpecId::TANGERINE,
            OptimismSpecId::SPURIOUS_DRAGON => SpecId::SPURIOUS_DRAGON,
            OptimismSpecId::BYZANTIUM => SpecId::BYZANTIUM,
            OptimismSpecId::CONSTANTINOPLE => SpecId::CONSTANTINOPLE,
            OptimismSpecId::PETERSBURG => SpecId::PETERSBURG,
            OptimismSpecId::ISTANBUL => SpecId::ISTANBUL,
            OptimismSpecId::MUIR_GLACIER => SpecId::MUIR_GLACIER,
            OptimismSpecId::BERLIN => SpecId::BERLIN,
            OptimismSpecId::LONDON => SpecId::LONDON,
            OptimismSpecId::ARROW_GLACIER => SpecId::ARROW_GLACIER,
            OptimismSpecId::GRAY_GLACIER => SpecId::GRAY_GLACIER,
            OptimismSpecId::MERGE | OptimismSpecId::BEDROCK | OptimismSpecId::REGOLITH => {
                SpecId::MERGE
            }
            OptimismSpecId::SHANGHAI | OptimismSpecId::CANYON => SpecId::SHANGHAI,
            OptimismSpecId::CANCUN | OptimismSpecId::ECOTONE | OptimismSpecId::FJORD => {
                SpecId::CANCUN
            }
            OptimismSpecId::PRAGUE => SpecId::PRAGUE,
            OptimismSpecId::PRAGUE_EOF => SpecId::PRAGUE_EOF,
            OptimismSpecId::LATEST => SpecId::LATEST,
        }
    }
}

impl From<OptimismSpecId> for SpecId {
    fn from(value: OptimismSpecId) -> Self {
        value.into_eth_spec_id()
    }
}

impl From<SpecId> for OptimismSpecId {
    fn from(value: SpecId) -> Self {
        match value {
            SpecId::FRONTIER => Self::FRONTIER,
            SpecId::FRONTIER_THAWING => Self::FRONTIER_THAWING,
            SpecId::HOMESTEAD => Self::HOMESTEAD,
            SpecId::DAO_FORK => Self::DAO_FORK,
            SpecId::TANGERINE => Self::TANGERINE,
            SpecId::SPURIOUS_DRAGON => Self::SPURIOUS_DRAGON,
            SpecId::BYZANTIUM => Self::BYZANTIUM,
            SpecId::CONSTANTINOPLE => Self::CONSTANTINOPLE,
            SpecId::PETERSBURG => Self::PETERSBURG,
            SpecId::ISTANBUL => Self::ISTANBUL,
            SpecId::MUIR_GLACIER => Self::MUIR_GLACIER,
            SpecId::BERLIN => Self::BERLIN,
            SpecId::LONDON => Self::LONDON,
            SpecId::ARROW_GLACIER => Self::ARROW_GLACIER,
            SpecId::GRAY_GLACIER => Self::GRAY_GLACIER,
            SpecId::MERGE => Self::MERGE,
            SpecId::SHANGHAI => Self::SHANGHAI,
            SpecId::CANCUN => Self::CANCUN,
            SpecId::PRAGUE => Self::PRAGUE,
            SpecId::PRAGUE_EOF => Self::PRAGUE_EOF,
            SpecId::LATEST => Self::LATEST,
        }
    }
}

impl From<OptimismSpecId> for PrecompileSpecId {
    fn from(value: OptimismSpecId) -> Self {
        PrecompileSpecId::from_spec_id(value.into_eth_spec_id())
    }
}

impl From<&str> for OptimismSpecId {
    fn from(name: &str) -> Self {
        match name {
            "Frontier" => Self::FRONTIER,
            "Homestead" => Self::HOMESTEAD,
            "Tangerine" => Self::TANGERINE,
            "Spurious" => Self::SPURIOUS_DRAGON,
            "Byzantium" => Self::BYZANTIUM,
            "Constantinople" => Self::CONSTANTINOPLE,
            "Petersburg" => Self::PETERSBURG,
            "Istanbul" => Self::ISTANBUL,
            "MuirGlacier" => Self::MUIR_GLACIER,
            "Berlin" => Self::BERLIN,
            "London" => Self::LONDON,
            "Merge" => Self::MERGE,
            "Shanghai" => Self::SHANGHAI,
            "Cancun" => Self::CANCUN,
            "Prague" => Self::PRAGUE,
            "PragueEOF" => Self::PRAGUE_EOF,
            "Bedrock" => Self::BEDROCK,
            "Regolith" => Self::REGOLITH,
            "Canyon" => Self::CANYON,
            "Ecotone" => Self::ECOTONE,
            "Fjord" => Self::FJORD,
            _ => Self::LATEST,
        }
    }
}

impl From<OptimismSpecId> for &'static str {
    fn from(value: OptimismSpecId) -> Self {
        match value {
            OptimismSpecId::FRONTIER
            | OptimismSpecId::FRONTIER_THAWING
            | OptimismSpecId::HOMESTEAD
            | OptimismSpecId::DAO_FORK
            | OptimismSpecId::TANGERINE
            | OptimismSpecId::SPURIOUS_DRAGON
            | OptimismSpecId::BYZANTIUM
            | OptimismSpecId::CONSTANTINOPLE
            | OptimismSpecId::PETERSBURG
            | OptimismSpecId::ISTANBUL
            | OptimismSpecId::MUIR_GLACIER
            | OptimismSpecId::BERLIN
            | OptimismSpecId::LONDON
            | OptimismSpecId::ARROW_GLACIER
            | OptimismSpecId::GRAY_GLACIER
            | OptimismSpecId::MERGE
            | OptimismSpecId::SHANGHAI
            | OptimismSpecId::CANCUN
            | OptimismSpecId::PRAGUE
            | OptimismSpecId::PRAGUE_EOF => value.into_eth_spec_id().into(),
            OptimismSpecId::BEDROCK => "Bedrock",
            OptimismSpecId::REGOLITH => "Regolith",
            OptimismSpecId::CANYON => "Canyon",
            OptimismSpecId::ECOTONE => "Ecotone",
            OptimismSpecId::FJORD => "Fjord",
            OptimismSpecId::LATEST => "Latest",
        }
    }
}

pub trait OptimismSpec: Spec + Sized + 'static {
    /// The specification ID for optimism.
    const OPTIMISM_SPEC_ID: OptimismSpecId;

    /// Returns whether the provided `OptimismSpec` is enabled by this spec.
    #[inline]
    fn optimism_enabled(spec_id: OptimismSpecId) -> bool {
        OptimismSpecId::enabled(Self::OPTIMISM_SPEC_ID, spec_id)
    }
}

macro_rules! spec {
    ($spec_id:ident, $spec_name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $spec_name;

        impl OptimismSpec for $spec_name {
            const OPTIMISM_SPEC_ID: OptimismSpecId = OptimismSpecId::$spec_id;
        }

        impl Spec for $spec_name {
            const SPEC_ID: SpecId = $spec_name::OPTIMISM_SPEC_ID.into_eth_spec_id();
        }
    };
}

spec!(FRONTIER, FrontierSpec);
// FRONTIER_THAWING no EVM spec change
spec!(HOMESTEAD, HomesteadSpec);
// DAO_FORK no EVM spec change
spec!(TANGERINE, TangerineSpec);
spec!(SPURIOUS_DRAGON, SpuriousDragonSpec);
spec!(BYZANTIUM, ByzantiumSpec);
// CONSTANTINOPLE was overridden with PETERSBURG
spec!(PETERSBURG, PetersburgSpec);
spec!(ISTANBUL, IstanbulSpec);
// MUIR_GLACIER no EVM spec change
spec!(BERLIN, BerlinSpec);
spec!(LONDON, LondonSpec);
// ARROW_GLACIER no EVM spec change
// GRAY_GLACIER no EVM spec change
spec!(MERGE, MergeSpec);
spec!(SHANGHAI, ShanghaiSpec);
spec!(CANCUN, CancunSpec);
spec!(PRAGUE, PragueSpec);
spec!(PRAGUE_EOF, PragueEofSpec);

spec!(LATEST, LatestSpec);

// Optimism Hardforks
spec!(BEDROCK, BedrockSpec);
spec!(REGOLITH, RegolithSpec);
spec!(CANYON, CanyonSpec);
spec!(ECOTONE, EcotoneSpec);
spec!(FJORD, FjordSpec);

#[macro_export]
macro_rules! optimism_spec_to_generic {
    ($spec_id:expr, $e:expr) => {{
        // We are transitioning from var to generic spec.
        match $spec_id {
            $crate::optimism::OptimismSpecId::FRONTIER
            | $crate::optimism::OptimismSpecId::FRONTIER_THAWING => {
                use $crate::optimism::FrontierSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::HOMESTEAD
            | $crate::optimism::OptimismSpecId::DAO_FORK => {
                use $crate::optimism::HomesteadSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::TANGERINE => {
                use $crate::optimism::TangerineSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::SPURIOUS_DRAGON => {
                use $crate::optimism::SpuriousDragonSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::BYZANTIUM => {
                use $crate::optimism::ByzantiumSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::PETERSBURG
            | $crate::optimism::OptimismSpecId::CONSTANTINOPLE => {
                use $crate::optimism::PetersburgSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::ISTANBUL
            | $crate::optimism::OptimismSpecId::MUIR_GLACIER => {
                use $crate::optimism::IstanbulSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::BERLIN => {
                use $crate::optimism::BerlinSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::LONDON
            | $crate::optimism::OptimismSpecId::ARROW_GLACIER
            | $crate::optimism::OptimismSpecId::GRAY_GLACIER => {
                use $crate::optimism::LondonSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::MERGE => {
                use $crate::optimism::MergeSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::SHANGHAI => {
                use $crate::optimism::ShanghaiSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::CANCUN => {
                use $crate::optimism::CancunSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::LATEST => {
                use $crate::optimism::LatestSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::PRAGUE => {
                use $crate::optimism::PragueSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::PRAGUE_EOF => {
                use $crate::optimism::PragueEofSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::BEDROCK => {
                use $crate::optimism::BedrockSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::REGOLITH => {
                use $crate::optimism::RegolithSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::CANYON => {
                use $crate::optimism::CanyonSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::ECOTONE => {
                use $crate::optimism::EcotoneSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::FJORD => {
                use $crate::optimism::FjordSpec as SPEC;
                $e
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimism_spec_to_generic() {
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER_THAWING,
            assert_eq!(SPEC::SPEC_ID, SpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::HOMESTEAD,
            assert_eq!(SPEC::SPEC_ID, SpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::DAO_FORK,
            assert_eq!(SPEC::SPEC_ID, SpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::TANGERINE,
            assert_eq!(SPEC::SPEC_ID, SpecId::TANGERINE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SPURIOUS_DRAGON,
            assert_eq!(SPEC::SPEC_ID, SpecId::SPURIOUS_DRAGON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BYZANTIUM,
            assert_eq!(SPEC::SPEC_ID, SpecId::BYZANTIUM)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CONSTANTINOPLE,
            assert_eq!(SPEC::SPEC_ID, SpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PETERSBURG,
            assert_eq!(SPEC::SPEC_ID, SpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ISTANBUL,
            assert_eq!(SPEC::SPEC_ID, SpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MUIR_GLACIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BERLIN,
            assert_eq!(SPEC::SPEC_ID, SpecId::BERLIN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LONDON,
            assert_eq!(SPEC::SPEC_ID, SpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ARROW_GLACIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::GRAY_GLACIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MERGE,
            assert_eq!(SPEC::SPEC_ID, SpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BEDROCK,
            assert_eq!(SPEC::SPEC_ID, SpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::REGOLITH,
            assert_eq!(SPEC::SPEC_ID, SpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SHANGHAI,
            assert_eq!(SPEC::SPEC_ID, SpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANYON,
            assert_eq!(SPEC::SPEC_ID, SpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANCUN,
            assert_eq!(SPEC::SPEC_ID, SpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ECOTONE,
            assert_eq!(SPEC::SPEC_ID, SpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FJORD,
            assert_eq!(SPEC::SPEC_ID, SpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE,
            assert_eq!(SPEC::SPEC_ID, SpecId::PRAGUE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LATEST,
            assert_eq!(SPEC::SPEC_ID, SpecId::LATEST)
        );

        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER_THAWING,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::HOMESTEAD,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::DAO_FORK,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::TANGERINE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::TANGERINE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SPURIOUS_DRAGON,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::SPURIOUS_DRAGON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BYZANTIUM,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::BYZANTIUM)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CONSTANTINOPLE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PETERSBURG,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ISTANBUL,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MUIR_GLACIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BERLIN,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::BERLIN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LONDON,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ARROW_GLACIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::GRAY_GLACIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MERGE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BEDROCK,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::BEDROCK)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::REGOLITH,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::REGOLITH)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SHANGHAI,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANYON,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::CANYON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANCUN,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ECOTONE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::ECOTONE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FJORD,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::FJORD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PRAGUE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE_EOF,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PRAGUE_EOF)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LATEST,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LATEST)
        );
    }

    #[test]
    fn test_bedrock_post_merge_hardforks() {
        assert!(BedrockSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(BedrockSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::REGOLITH));
    }

    #[test]
    fn test_regolith_post_merge_hardforks() {
        assert!(RegolithSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(!RegolithSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(!RegolithSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!RegolithSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(RegolithSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(RegolithSpec::optimism_enabled(OptimismSpecId::REGOLITH));
    }

    #[test]
    fn test_bedrock_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::MERGE
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::SHANGHAI
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::BEDROCK
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::REGOLITH
        ));
    }

    #[test]
    fn test_regolith_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::MERGE
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::SHANGHAI
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::REGOLITH
        ));
    }

    #[test]
    fn test_canyon_post_merge_hardforks() {
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(!CanyonSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!CanyonSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::REGOLITH));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::CANYON));
    }

    #[test]
    fn test_canyon_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::MERGE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::SHANGHAI
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::REGOLITH
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::CANYON
        ));
    }

    #[test]
    fn test_ecotone_post_merge_hardforks() {
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!EcotoneSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::REGOLITH));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::CANYON));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::ECOTONE));
    }

    #[test]
    fn test_ecotone_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::MERGE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::SHANGHAI
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::REGOLITH
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::CANYON
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::ECOTONE
        ));
    }

    #[test]
    fn test_fjord_post_merge_hardforks() {
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!FjordSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::REGOLITH));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::CANYON));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::ECOTONE));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::FJORD));
    }

    #[test]
    fn test_fjord_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::MERGE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::SHANGHAI
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::REGOLITH
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::CANYON
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::ECOTONE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::FJORD
        ));
    }
}
