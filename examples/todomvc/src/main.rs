use kobold::prelude::*;
use web_sys::HtmlInputElement;

mod filter;
mod state;

use filter::Filter;
use state::*;

struct App;

impl Stateful for App {
    type State = State;

    fn init(self) -> State {
        State {
            entries: Vec::new(),
            filter: Filter::All,
            editing: None,
        }
    }

    fn update(self, _: &mut State) -> ShouldRender {
        // App is rendered only once
        ShouldRender::No
    }
}

impl App {
    fn render(self) -> impl Html {
        self.stateful(|state, ctx| {
            let (main_class, footer_class) = if state.entries.is_empty() {
                ("main hidden", "main hidden")
            } else {
                ("main", "footer")
            };

            let active_count = state.count_active();
            let completed_count = state.entries.len() - active_count;
            let is_all_completed = active_count == 0;
            let selected = state.filter;

            html! {
                <div .todomvc-wrapper>
                    <section .todoapp>
                        <header .header>
                            <h1>"todos"</h1>
                            <EntryInput {ctx} />
                        </header>
                        <section .{main_class}>
                            {
                                html! {
                                    <input
                                        #toggle-all
                                        .toggle-all
                                        type="checkbox"
                                        checked={is_all_completed}
                                        onclick={ctx.bind(move |state, _| state.set_all(!is_all_completed))}
                                    />
                                }
                            }
                            <label for="toggle-all" />
                            <ul .todo-list>
                            {
                                state
                                    .filtered_entries()
                                    .map(move |(idx, entry)| html! { <EntryView {idx} {entry} {ctx} /> })
                                    .list()
                            }
                            </ul>
                        </section>
                        <footer .{footer_class}>
                            <span .todo-count>
                                <strong>{ active_count }</strong>
                                { if active_count == 1 { " item left" } else { " items left" } }
                            </span>
                            <ul .filters>
                                <FilterView filter={Filter::All} {selected} {ctx} />
                                <FilterView filter={Filter::Active} {selected} {ctx} />
                                <FilterView filter={Filter::Completed} {selected} {ctx} />
                            </ul>
                            <button .clear-completed onclick={ctx.bind(|state, _| state.entries.retain(|entry| !entry.completed))}>
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
}

struct EntryInput<'a> {
    ctx: Context<'a, State>,
}

mod wat {
    use super::*;

    impl<'a> EntryInput<'a> {
        pub fn render(self) -> impl Html + 'a {
            html! {
                <input
                    .new-todo
                    placeholder="What needs to be done?"
                    onchange={self.ctx.bind(|state, event| {
                        let input = event.target();
                        let value = input.value();

                        input.set_value("");
                        state.add(value);

                        ShouldRender::Yes
                    })}
                />
            }
        }
    }
}

struct EntryView<'a> {
    idx: usize,
    entry: &'a Entry,
    ctx: Context<'a, State>,
}

fn test(checked: bool) -> impl Html {
    html! { <p><input {checked} value="foo" width={ 42 } /></p> }
}

impl<'a> EntryView<'a> {
    fn render(self) -> impl Html + 'a {
        let EntryView { idx, entry, ctx } = self;

        let class = match (entry.editing, entry.completed) {
            (false, false) => "todo",
            (true, false) => "todo editing",
            (false, true) => "todo completed",
            (true, true) => "todo editing completed",
        };

        let input = self.entry.editing.then(move || {
            let onmouseover = ctx.bind(move |_, event| {
                let input: HtmlInputElement = event.target();

                if input.focus().is_ok() {
                    input.select();
                }

                ShouldRender::No
            });

            html! {
                <input .edit
                    type="text"
                    value={&self.entry.description}
                    {onmouseover}
                    onchange={ctx.bind(move |state, event| state.update(idx, event.target().value()))}
                />
            }
        });

        let onchange = ctx.bind(move |state, _| state.toggle(idx));

        html! {
            <li {class}>
                <div .view>
                    {
                        html! {
                            <input .toggle type="checkbox" checked={entry.completed} {onchange} />
                        }
                    }
                    <label ondblclick={ctx.bind(move |state, _| state.edit_entry(idx))} >
                        { &entry.description }
                    </label>
                    <button .destroy onclick={ctx.bind(move |state, _| state.remove(idx))} />
                </div>
                { input }
            </li>
        }
    }
}

struct FilterView<'a> {
    filter: Filter,
    selected: Filter,
    ctx: Context<'a, State>,
}

impl<'a> FilterView<'a> {
    fn render(self) -> impl Html + 'a {
        let filter = self.filter;
        let class = if self.selected == filter {
            "selected"
        } else {
            "not-selected"
        };
        let href = filter.to_href();
        let onclick = self.ctx.bind(move |state, _| state.filter = filter);

        html! {
            <li>
                <a {class} {href} {onclick}>{ filter.to_label() }</a>
            </li>
        }
    }
}

fn main() {
    kobold::start(html! {
        <App />
    });
}
