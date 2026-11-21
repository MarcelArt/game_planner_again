use gpui_kit::component::color_picker::{ColorPickerEvent, ColorSelect};
use gpui_kit::component::scroll::ScrollableElement as _;

use super::*;
use crate::appearance::Preset;

impl Planner {
    fn sync_color_fields(&self, window: &mut Window, cx: &mut Context<Self>) {
        let colors = self.appearance.colors();
        for (field, value) in self
            .color_fields
            .iter()
            .zip([colors.background, colors.foreground, colors.accent])
        {
            field.update(cx, |state, cx| state.set_value(rgb(value), window, cx));
        }
    }

    fn change_appearance(&mut self, next: Appearance, window: &mut Window, cx: &mut Context<Self>) {
        if next != self.appearance {
            match self.db.save_appearance(next) {
                Ok(()) => {
                    self.appearance = next;
                    apply_theme(next, cx);
                }
                Err(err) => self.notify_error(&err, window, cx),
            }
        }
        self.sync_color_fields(window, cx);
        cx.notify();
    }

    pub(super) fn edit_color(
        &mut self,
        index: usize,
        event: &ColorPickerEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ColorPickerEvent::Change(Some(value)) = event else {
            self.sync_color_fields(window, cx);
            return;
        };
        let rgb = value.to_rgb();
        let byte = |v: f32| (v * 255.).round() as u32;
        let value = byte(rgb.r) << 16 | byte(rgb.g) << 8 | byte(rgb.b);
        let mut colors = self.appearance.colors();
        let role = match index {
            0 => &mut colors.background,
            1 => &mut colors.foreground,
            _ => &mut colors.accent,
        };
        if *role == value {
            self.sync_color_fields(window, cx);
            return;
        }
        *role = value;
        let mut next = self.appearance;
        next.custom[usize::from(next.light)] = Some(colors);
        next.preset = Preset::Custom;
        self.change_appearance(next, window, cx);
    }

    pub(super) fn settings_page(&self, cx: &mut Context<Self>) -> Div {
        let appearance = self.appearance;
        let mode = h_flex()
            .gap(px(3.))
            .p(px(3.))
            .rounded(px(9.))
            .bg(rgb(pal::well()))
            .children(
                [
                    (true, "theme-light", "Light mode", Lucide::Sun),
                    (false, "theme-dark", "Dark mode", Lucide::Moon),
                ]
                .into_iter()
                .map(|(light, id, label, icon)| {
                    Button::new(id)
                        .ghost()
                        .icon(Icon::new(icon))
                        .selected(light == appearance.light)
                        .toggled(light == appearance.light)
                        .tooltip(label)
                        .accessibility_label(label)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.change_appearance(
                                Appearance {
                                    light,
                                    ..this.appearance
                                },
                                window,
                                cx,
                            );
                        }))
                }),
            );
        let mut themes = v_flex().gap(px(8.));
        for row in Preset::ALL.chunks(3) {
            themes = themes.child(h_flex().gap(px(8.)).children(row.iter().map(|&preset| {
                let selected = preset == appearance.preset;
                let colors = Appearance { preset, ..appearance }.colors();
                let swatches =
                    h_flex()
                        .gap(px(4.))
                        .children([colors.background, colors.foreground, colors.accent].map(|color| {
                            div()
                                .size(px(12.))
                                .rounded_full()
                                .bg(rgb(color))
                                .border_1()
                                .border_color(rgb(pal::edge_2()))
                        }));
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .h(px(46.))
                    .rounded(px(8.))
                    .bg(rgb(if selected { pal::accent_fill() } else { pal::tile() }))
                    .border_1()
                    .border_color(rgb(if selected { pal::accent() } else { pal::edge() }))
                    .child(
                        Button::new(preset.name())
                            .size_full()
                            .ghost()
                            .accessibility_label(preset.name())
                            .toggled(selected)
                            .child(h_flex().w_full().gap(px(10.)).child(swatches).child(preset.name()))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.change_appearance(
                                    Appearance {
                                        preset,
                                        ..this.appearance
                                    },
                                    window,
                                    cx,
                                );
                            })),
                    )
            })));
        }
        let fields = h_flex().gap(px(12.)).children(
            ["Background", "Foreground", "Accent"]
                .into_iter()
                .zip(&self.color_fields)
                .map(|(label, field)| {
                    v_flex()
                        .flex_1()
                        .gap(px(6.))
                        .child(label)
                        .child(ColorSelect::new(field).accessibility_label(label))
                }),
        );
        v_flex().size_full().child(
            v_flex().flex_1().min_h_0().overflow_y_scrollbar().p(px(40.)).child(
                v_flex()
                    .w_full()
                    .max_w(px(760.))
                    .gap(px(24.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(FontWeight::EXTRA_BOLD)
                            .child("Settings"),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .child(section_title("Appearance", ""))
                            .child(mode),
                    )
                    .child(v_flex().gap(px(10.)).child("Theme").child(themes))
                    .child(
                        v_flex()
                            .gap(px(10.))
                            .child(section_title("Custom colors", ""))
                            .child(
                                div()
                                    .text_color(rgb(pal::muted()))
                                    .child("Editing a color saves a custom theme for this mode."),
                            )
                            .child(fields),
                    ),
            ),
        )
    }
}
