use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Tutorial {
    Library,
    Waiting,
    Intro,
    Items,
    Inventory,
    Plans,
    Requirements,
    Off,
}

impl Tutorial {
    pub(super) fn target(self) -> Page {
        match self {
            Self::Library | Self::Waiting | Self::Off => Page::Library,
            Self::Intro | Self::Items => Page::Items,
            Self::Inventory => Page::Inventory,
            Self::Plans => Page::Plans,
            Self::Requirements => Page::Requirements,
        }
    }
}

impl Planner {
    pub(super) fn set_tutorial(
        &mut self,
        step: Tutorial,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match self.db.save_tutorial(step) {
            Ok(()) => {
                self.tutorial = step;
                cx.notify();
                true
            }
            Err(err) => {
                self.notify_error(&err, window, cx);
                false
            }
        }
    }

    fn waiting_for_game(&self) -> bool {
        self.game.is_none() && self.tutorial != Tutorial::Library
    }

    pub(super) fn tutorial_target(&self) -> Option<Page> {
        if self.tutorial == Tutorial::Off || self.page == Page::Settings {
            None
        } else if self.waiting_for_game() {
            Some(Page::Library)
        } else {
            Some(self.tutorial.target())
        }
    }

    fn advance_tutorial(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use Tutorial::*;
        let next = match self.tutorial {
            Library if self.game.is_none() => Waiting,
            Library | Waiting => Intro,
            Intro => Items,
            Items => Inventory,
            Inventory => Plans,
            Plans => Requirements,
            Requirements => Off,
            Off => return,
        };
        if self.set_tutorial(next, window, cx) && next != Off {
            self.go(next.target(), cx);
        }
    }

    pub(super) fn tutorial_bubble(&self, cx: &mut Context<Self>) -> Div {
        use Tutorial::*;
        let waiting = self.waiting_for_game() || self.tutorial == Waiting;
        let message = if waiting {
            "Open a game to continue. You can also add a new game or import a game file."
        } else {
            match self.tutorial {
                Library => {
                    "Start in the game library. Add a game, import a game file, or open a game you already have."
                }
                Intro => "Open Items to see what this game lets you gather and craft.",
                Items => {
                    "Use Add item to name a material or craftable item. Select it to describe its recipe inputs and output."
                }
                Inventory => {
                    "Inventory records what you own. Enter a quantity on an item and press Enter to update your requirements."
                }
                Plans => {
                    "Choose an item and a target amount, then use Add plan to record your goal."
                }
                Requirements => {
                    "Read all your plans together here. Gather shows missing materials. Craft shows what to make, and Ready shows what you can craft now."
                }
                Waiting | Off => unreachable!(),
            }
        };
        v_flex()
            .gap(px(6.))
            .p(px(10.))
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(pal::accent()))
            .bg(rgb(pal::tile()))
            .text_size(px(13.))
            .text_color(rgb(pal::text()))
            .child(message)
            .when(!waiting, |el| {
                el.child(
                    Button::new("tutorial-next")
                        .primary()
                        .label(if self.tutorial == Requirements {
                            "Finish"
                        } else {
                            "Next"
                        })
                        .on_click(
                            cx.listener(|this, _, window, cx| this.advance_tutorial(window, cx)),
                        ),
                )
            })
            .child(
                Button::new("tutorial-skip")
                    .ghost()
                    .label("Skip tutorial")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.set_tutorial(Off, window, cx);
                    })),
            )
    }
}
