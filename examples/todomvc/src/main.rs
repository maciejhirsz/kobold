use kobold::prelude::*;
use web_sys::HtmlInputElement as InputElement;

mod filter;
mod state;

use filter::Filter;
use state::*;

#[component]
fn App() -> impl Html {
    stateful(State::load, |state| {
        let hidden = state.entries.is_empty().then_some("hidden");

        let active_count = state.count_active();
        let completed_count = state.entries.len() - active_count;
        let is_all_completed = active_count == 0;
        let selected = state.filter;

        html! {
            <div .todomvc-wrapper>
                <section .todoapp>
                    <header .header>
                        <h1>"todos"</h1>
                        <EntryInput {state} />
                    </header>
                    <section .main.{hidden}>
                        <input
                            #toggle-all
                            .toggle-all
                            type="checkbox"
                            checked={is_all_completed}
                            onclick={state.bind(move |state, _| state.set_all(!is_all_completed))}
                        />
                        <label for="toggle-all" />
                        <ul .todo-list>
                            {
                                state
                                    .filtered_entries()
                                    .map(move |(idx, entry)| html! { <EntryView {idx} {entry} {state} /> })
                                    .list()
                            }
                        </ul>
                    </section>
                    <footer .footer.{hidden}>
                        <span .todo-count>
                            <strong>{ active_count }</strong>
                            { if active_count == 1 { " item left" } else { " items left" } }
                        </span>
                        <ul .filters>
                            <FilterView filter={Filter::All} {selected} {state} />
                            <FilterView filter={Filter::Active} {selected} {state} />
                            <FilterView filter={Filter::Completed} {selected} {state} />
                        </ul>
                        <button .clear-completed onclick={state.bind(|state, _| state.entries.retain(|entry| !entry.completed))}>
                            "Clear completed ("{ completed_count }")"
                        </button>
                    </footer>
                </section>
                <footer .info>
                    <p>"Double-click to edit a todo"</p>
                    <p>"Written by "<a href="https://maciej.codes/" target="_blank">"Maciej Hirsz"</a></p>
                    <p>"Part of "<a href="http://todomvc.com/" target="_blank">"TodoMVC"</a></p>
                </footer>
            </div>
        }
    })
}

#[component]
fn EntryInput(state: &Hook<State>) -> impl Html {
    html! {
        <input
            .new-todo
            placeholder="What needs to be done?"
            onchange={state.bind(|state, event| {
                let input = event.target();
                let value = input.value();

                input.set_value("");
                state.add(value);
            })}
        />
    }
}

#[component]
fn EntryView(idx: usize, entry: &Entry, state: &Hook<State>) -> impl Html {
    let input = entry.editing.then(move || {
        let onmouseover = state.bind(move |_, event: &MouseEvent<InputElement>| {
            let _ = event.target().focus();

            ShouldRender::No
        });

        let onkeypress = state.bind(move |state, event: &KeyboardEvent<InputElement>| {
            if event.key() == "Enter" {
                state.update(idx, event.target().value());

                ShouldRender::Yes
            } else {
                ShouldRender::No
            }
        });

        html! {
            <input .edit
                type="text"
                value={&entry.description}
                {onmouseover}
                {onkeypress}
                onblur={state.bind(move |state, event| state.update(idx, event.target().value()))}
            />
        }
    });

    let onchange = state.bind(move |state, _| state.toggle(idx));
    let editing = entry.editing.then_some("editing");
    let completed = entry.completed.then_some("completed");

    html! {
        <li .todo.{editing}.{completed}>
            <div .view>
                <input .toggle type="checkbox" checked={entry.completed} {onchange} />
                <label ondblclick={state.bind(move |state, _| state.edit_entry(idx))} >
                    { &entry.description }
                </label>
                <button .destroy onclick={state.bind(move |state, _| state.remove(idx))} />
            </div>
            { input }
        </li>
    }
}

#[component]
fn FilterView(filter: Filter, selected: Filter, state: &Hook<State>) -> impl Html {
    let class = if selected == filter {
        "selected"
    } else {
        "not-selected"
    };
    let href = filter.to_href();
    let onclick = state.bind(move |state, _| state.filter = filter);

    html! {
        <li>
            <a {class} {href} {onclick}>{ filter.to_label() }</a>
        </li>
    }
}

fn main() {
    kobold::start(html! {
        <App />
    });
}
