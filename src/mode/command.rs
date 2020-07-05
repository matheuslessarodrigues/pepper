use crate::{
    command::CommandContext,
    editor::Operation,
    mode::{poll_input, FromMode, InputResult, ModeContext},
};

pub fn on_enter(ctx: ModeContext) {
    ctx.input.clear();
}

pub fn on_event(mut ctx: ModeContext, from_mode: &FromMode) -> Operation {
    match poll_input(&mut ctx) {
        InputResult::Canceled => Operation::EnterMode(from_mode.as_mode()),
        InputResult::Submited => {
            let command_name;
            let command_args;
            if let Some(index) = ctx.input.find(' ') {
                command_name = &ctx.input[..index];
                command_args = &ctx.input[index..];
            } else {
                command_name = &ctx.input[..];
                command_args = "";
            }

            let command_context = CommandContext {
                buffers: ctx.buffers,
                buffer_views: ctx.buffer_views,
                viewports: ctx.viewports,
            };

            ctx.commands
                .execute(command_name, command_context, command_args)
        }
        InputResult::Pending => Operation::None,
    }
}
