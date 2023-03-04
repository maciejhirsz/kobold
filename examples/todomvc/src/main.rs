use kobold::prelude::*;
use web_sys::HtmlInputElement as InputElement;

mod filter;
mod state;

use filter::Filter;
use state::*;

#[component]
fn App() -> impl Html {
    stateful(State::default, |state| {
        let hidden = state.entries.is_empty().class("hidden").no_diff();

        let active_count = state.count_active();
        let completed_hidden = (state.entries.len() == active_count)
            .class("hidden")
            .no_diff();

        let clear = state.bind(|state, _| state.entries.retain(|entry| !entry.completed));

        html! {
            <div .todomvc-wrapper>
                <section .todoapp>
                    <header .header>
                        <h1>"todos"</h1>
                        <EntryInput {state} />
                    </header>
                    <section .main.{hidden}>
                        <ToggleAll {active_count} {state} />
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
                            {
                                match active_count {
                                    1 => " item left",
                                    _ => " items left",
                                }
                                .fast_diff()
                            }
                        </span>
                        <ul .filters>
                            <FilterView filter={Filter::All} {state} />
                            <FilterView filter={Filter::Active} {state} />
                            <FilterView filter={Filter::Completed} {state} />
                        </ul>
                        <button .clear-completed.{completed_hidden} onclick={clear}>
                            "Clear completed"
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
fn EntryInput(state: &Hook<State>) -> impl Html + '_ {
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
fn ToggleAll(active_count: usize, state: &Hook<State>) -> impl Html + '_ {
    let onclick = state.bind(move |state, _| state.set_all(active_count != 0));

    html! {
        <input #toggle-all.toggle-all type="checkbox" checked={active_count == 0} onclick={onclick} />
        <label for="toggle-all" />
    }
}

#[component]
fn EntryView<'a>(idx: usize, entry: &'a Entry, state: &'a Hook<State>) -> impl Html + 'a {
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
                value={entry.description.fast_diff()}
                {onmouseover}
                {onkeypress}
                onblur={state.bind(move |state, event| state.update(idx, event.target().value()))}
            />
        }
    });

    let onchange = state.bind(move |state, _| state.toggle(idx));
    let editing = entry.editing.class("editing").no_diff();
    let completed = entry.completed.class("completed").no_diff();

    html! {
        <li .todo.{editing}.{completed}>
            <div .view>
                <input .toggle type="checkbox" checked={entry.completed} {onchange} />
                <label ondblclick={state.bind(move |state, _| state.edit_entry(idx))} >
                    { entry.description.fast_diff() }
                </label>
                <button .destroy onclick={state.bind(move |state, _| state.remove(idx))} />
            </div>
            { input }
        </li>
    }
}

#[component]
fn FilterView(filter: Filter, state: &Hook<State>) -> impl Html + '_ {
    let selected = state.filter;

    let class = (selected == filter).class("selected").no_diff();
    let href = filter.href().no_diff();
    let onclick = state.bind(move |state, _| state.filter = filter);

    html! {
        <li>
            <a {class} {href} {onclick}>{ filter.label().no_diff() }</a>
        </li>
    }
}

fn main() {
    kobold::start(html! {
        <App />
    });
}
