//! Wizard stepper component for step-by-step navigation
//!
//! Displays a horizontal progress indicator showing:
//! - Current step (active/highlighted)
//! - Completed steps (with checkmark)
//! - Pending steps (grayed out)
//!
//! Users can click on accessible steps to navigate directly.

use yew::prelude::*;
use crate::hooks::WizardStep;

#[derive(Properties, PartialEq)]
pub struct WizardStepperProps {
    pub current_step: WizardStep,
    pub steps_completed: std::collections::HashMap<WizardStep, bool>,
    pub on_step_click: Callback<WizardStep>,
}

#[function_component(WizardStepper)]
pub fn wizard_stepper(props: &WizardStepperProps) -> Html {
    let steps = [WizardStep::SelectSkill,
        WizardStep::SelectTool,
        WizardStep::ConfigureParameters,
        WizardStep::Execute];

    html! {
        <div class="wizard-stepper">
            // Desktop: Horizontal stepper
            <div class="hidden md:flex items-center justify-between max-w-3xl mx-auto">
                { for steps.iter().enumerate().map(|(idx, step)| {
                    let is_current = props.current_step == *step;
                    let is_completed = props.steps_completed.get(step).copied().unwrap_or(false);
                    let is_pending = !is_current && !is_completed;

                    let step_clone = *step;
                    let on_click = {
                        let on_step_click = props.on_step_click.clone();
                        Callback::from(move |_| {
                            on_step_click.emit(step_clone);
                        })
                    };

                    html! {
                        <>
                            // Step circle with number/checkmark
                            <div class="flex flex-col items-center">
                                <button
                                    onclick={on_click}
                                    class={classes!(
                                        "wizard-step-circle",
                                        is_current.then_some("active"),
                                        is_completed.then_some("completed"),
                                        is_pending.then_some("pending"),
                                        "transition-all", "duration-200"
                                    )}
                                >
                                    if is_completed {
                                        // Checkmark icon
                                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                                        </svg>
                                    } else {
                                        // Step number
                                        { step.number() }
                                    }
                                </button>

                                // Step label
                                <span class={classes!(
                                    "text-xs", "font-medium", "mt-2", "text-center", "max-w-[100px]",
                                    if is_current {
                                        "text-primary-600 dark:text-primary-400"
                                    } else if is_completed {
                                        "text-green-600 dark:text-green-400"
                                    } else {
                                        "text-gray-400 dark:text-gray-500"
                                    }
                                )}>
                                    { step.label() }
                                </span>
                            </div>

                            // Connecting line (except after last step)
                            if idx < steps.len() - 1 {
                                <div class={classes!(
                                    "flex-1", "h-0.5", "mx-4", "transition-colors",
                                    if is_completed {
                                        "bg-green-500"
                                    } else {
                                        "bg-gray-300 dark:bg-gray-700"
                                    }
                                )}></div>
                            }
                        </>
                    }
                }) }
            </div>

            // Mobile: Vertical stepper
            <div class="md:hidden space-y-4">
                { for steps.iter().map(|step| {
                    let is_current = props.current_step == *step;
                    let is_completed = props.steps_completed.get(step).copied().unwrap_or(false);
                    let is_pending = !is_current && !is_completed;

                    let step_clone = *step;
                    let on_click = {
                        let on_step_click = props.on_step_click.clone();
                        Callback::from(move |_| {
                            on_step_click.emit(step_clone);
                        })
                    };

                    html! {
                        <button
                            onclick={on_click}
                            class={classes!(
                                "flex", "items-center", "gap-3", "w-full", "text-left",
                                "p-3", "rounded-lg", "transition-all",
                                if is_current {
                                    "bg-primary-50 dark:bg-primary-900/20 border-2 border-primary-500"
                                } else if is_completed {
                                    "bg-green-50 dark:bg-green-900/20 border-2 border-green-500"
                                } else {
                                    "bg-gray-50 dark:bg-gray-800 border-2 border-gray-300 dark:border-gray-700"
                                }
                            )}
                        >
                            <div class={classes!(
                                "wizard-step-circle",
                                is_current.then_some("active"),
                                is_completed.then_some("completed"),
                                is_pending.then_some("pending"),
                                "flex-shrink-0"
                            )}>
                                if is_completed {
                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                                    </svg>
                                } else {
                                    { step.number() }
                                }
                            </div>

                            <div class="flex-1">
                                <div class={classes!(
                                    "text-sm", "font-semibold",
                                    if is_current {
                                        "text-primary-600 dark:text-primary-400"
                                    } else if is_completed {
                                        "text-green-600 dark:text-green-400"
                                    } else {
                                        "text-gray-400 dark:text-gray-500"
                                    }
                                )}>
                                    { step.label() }
                                </div>

                                if is_completed {
                                    <div class="text-xs text-green-600 dark:text-green-400 mt-0.5">
                                        { "Completed" }
                                    </div>
                                } else if is_current {
                                    <div class="text-xs text-primary-600 dark:text-primary-400 mt-0.5">
                                        { "In Progress" }
                                    </div>
                                }
                            </div>

                            // Arrow indicator for current step
                            if is_current {
                                <svg class="w-5 h-5 text-primary-600 dark:text-primary-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                                </svg>
                            }
                        </button>
                    }
                }) }
            </div>
        </div>
    }
}
