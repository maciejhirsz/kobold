use kobold::prelude::*;

use web_sys::HtmlElement;

use crate::state::State;

#[component]
pub fn ButtonAddRow(row: usize, state: &Hook<State>) -> impl View {
    view! {
        <button.add
            data={row}
            onclick={
                state.bind(move |state, event: MouseEvent<HtmlElement>| {
                    let row = match event.target().get_attribute("data") {
                        Some(r) => r,
                        None => return,
                    };
                    let row_usize = match row.parse::<usize>() {
                        Ok(r) => r,
                        Err(e) => return,
                    };
                    // pass row index to insert at
                    state.add_row_main(row_usize);
                })
            }
        />
    }
}
