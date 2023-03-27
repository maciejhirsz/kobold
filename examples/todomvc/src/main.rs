use kobold::prelude::*;
use web_sys::HtmlInputElement as InputElement;

mod filter;
mod state;

use filter::Filter;
use state::*;

#[component]
fn App() -> impl View {
    stateful(State::default, |state| {
        let hidden = class!("hidden" if state.entries.is_empty());

        let active_count = state.count_active();
        let completed_hidden = class!("hidden" if state.entries.len() == active_count);

        bind! { state:
            let clear = move |_| state.clear();
        }

        view! {
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
                                for state
                                    .filtered_entries()
                                    .map(move |(idx, entry)| view! { <EntryView {idx} {entry} {state} /> })
                            }
                        </ul>
                    </section>
                    <footer .footer.{hidden}>
                        <span .todo-count>
                            <strong>{ active_count }</strong>
                            {
                                ref match active_count {
                                    1 => " item left",
                                    _ => " items left",
                                }
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
fn EntryInput(state: &Hook<State>) -> impl View + '_ {
    bind! { state:
        let onchange = move |event: Event<InputElement>| {
            let input = event.target();
            let value = input.value();

            input.set_value("");
            state.add(value);
        };
    }

    view! {
        <input.new-todo placeholder="What needs to be done?" {onchange} />
    }
}

#[component]
fn ToggleAll(active_count: usize, state: &Hook<State>) -> impl View + '_ {
    bind! { state:
        let onclick = move |_| state.set_all(active_count != 0);
    }

    view! {
        <input #toggle-all.toggle-all type="checkbox" checked={active_count == 0} {onclick} />
        <label for="toggle-all" />
    }
}

#[component]
fn EntryView<'a>(idx: usize, entry: &'a Entry, state: &'a Hook<State>) -> impl View + 'a {
    let input = entry.editing.then(move || {
        bind! { state:
            let onkeypress = move |event: KeyboardEvent<InputElement>| {
                if event.key() == "Enter" {
                    state.update(idx, event.target().value());

                    Then::Render
                } else {
                    Then::Stop
                }
            };

            let onblur = move |event: Event<InputElement>| state.update(idx, event.target().value());
        }

        let onmouseover = move |event: MouseEvent<InputElement>| {
            let _ = event.target().focus();
        };

        view! {
            <input .edit
                type="text"
                value={ref entry.description}
                {onmouseover}
                {onkeypress}
                {onblur}
            />
        }
    });

    bind! {
        state:
        let onchange = move |_| state.toggle(idx);
        let edit = move |_| state.edit_entry(idx);
        let remove = move |_| state.remove(idx);
    }
    let editing = class!("editing" if entry.editing);
    let completed = class!("completed" if entry.completed);

    view! {
        <li .todo.{editing}.{completed}>
            <div .view>
                <input .toggle type="checkbox" checked={entry.completed} {onchange} />
                <label ondblclick={edit} >
                    { ref entry.description }
                </label>
                <button .destroy onclick={remove} />
            </div>
            { input }
        </li>
    }
}

#[component]
fn FilterView(filter: Filter, state: &Hook<State>) -> impl View + '_ {
    let selected = state.filter;

    let class = class!("selected" if selected == filter);
    bind! { state:
        let onclick = move |_| state.filter = filter;
    }

    view! {
        <li>
            <a {class} {onclick} href={static filter.href()}>
            {
                static filter.label()
            }
            </a>
        </li>
    }
}

fn main() {
    kobold::start(view! {
        <App />
    });
}
