use crate::rules::{
    RequireMode, RequireModeLike, RobloxRequireMode, RuleProcessResult, SingularRequireMode, require::{HybridPathLocator, LuauPathLocator, LuauRequireMode, PathRequireMode, RequirePathLocator, RobloxPathLocator}
};
use crate::{nodes::Block, rules::Context};

use super::{path_require_mode, BundleOptions};

pub trait BundleRequireMode {
    fn process_block(
        &self,
        block: &mut Block,
        context: &Context,
        options: &BundleOptions,
    ) -> RuleProcessResult;
}

impl BundleRequireMode for PathRequireMode {
    fn process_block(
            &self,
            block: &mut Block,
            context: &Context,
            options: &BundleOptions,
        ) -> RuleProcessResult {
        let mut require_mode = self.clone();
        require_mode
            .initialize(context)
            .map_err(|err| err.to_string())?;

        let locator = RequirePathLocator::new(
            &require_mode,
            context.project_location(),
            context.resources(),
        );

        path_require_mode::process_block(block, context, options, locator)
    }
}

impl BundleRequireMode for LuauRequireMode {
    fn process_block(
            &self,
            block: &mut Block,
            context: &Context,
            options: &BundleOptions,
        ) -> RuleProcessResult {
        let mut require_mode = self.clone();
        require_mode
            .initialize(context)
            .map_err(|err| err.to_string())?;

        let locator = LuauPathLocator::new(
            &require_mode,
            context.project_location(),
            context.resources(),
        );

        path_require_mode::process_block(block, context, options, locator)
    }
}

impl BundleRequireMode for RobloxRequireMode {
    fn process_block(
            &self,
            block: &mut Block,
            context: &Context,
            options: &BundleOptions,
        ) -> RuleProcessResult {
        let mut require_mode = self.clone();
        require_mode
            .initialize(context)
            .map_err(|err| err.to_string())?;

        let locator = RobloxPathLocator::new(
            &require_mode,
            context.project_location(),
            context.resources()
        );

        path_require_mode::process_block(block, context, options, locator)
    }
}

impl BundleRequireMode for SingularRequireMode {
    fn process_block(
        &self,
        block: &mut Block,
        context: &Context,
        options: &BundleOptions,
    ) -> RuleProcessResult {
        match self {
            Self::Path(path_require_mode) => path_require_mode.process_block(block, context, options),
            Self::Luau(luau_require_mode) => luau_require_mode.process_block(block, context, options),
            Self::Roblox(roblox_require_mode) => roblox_require_mode.process_block(block, context, options)
        }
    }
}

impl BundleRequireMode for RequireMode {
    fn process_block(
        &self,
        block: &mut Block,
        context: &Context,
        options: &BundleOptions,
    ) -> RuleProcessResult {
        match self {
            RequireMode::Single(singular_require_mode) => singular_require_mode.process_block(block, context, options),
            RequireMode::Hybrid(singular_require_modes) => {
                let mut modes = singular_require_modes.clone();
                for mode in modes.iter_mut() {
                    mode.initialize(context).map_err(|err| err.to_string())?;
                }

                let locator = HybridPathLocator::new(
                    &modes,
                    context.project_location(),
                    context.resources(),
                );

                path_require_mode::process_block(block, context, options, locator)
            },
        }
    }
}