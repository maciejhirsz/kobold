use kobold::prelude::*;
use web_sys::HtmlInputElement;

struct App;

pub struct State {
    pub entries: Vec<Entry>,
    pub filter: Filter,
    pub editing: Option<usize>,
}

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

pub struct Entry {
    pub description: String,
    pub completed: bool,
    pub editing: bool,
}

impl Entry {
    fn filter(&self, f: Filter) -> bool {
        match f {
            Filter::All => true,
            Filter::Active => !self.completed,
            Filter::Completed => self.completed,
        }
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
    }

    fn update(&mut self, description: String) -> ShouldRender {
        self.editing = false;
        self.description = description;

        ShouldRender::Yes
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    fn to_href(self) -> &'static str {
        match self {
            Filter::All => "#/",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
        }
    }

    fn to_label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Active => "Active",
            Filter::Completed => "Completed",
        }
    }
}

impl App {
    fn render(self) -> impl Html {
        self.stateful(|state, link| {
            let (main_class, footer_class) = if state.entries.is_empty() {
                ("main hidden", "main hidden")
            } else {
                ("main", "footer")
            };

            let active_count = state.count_active();
            let completed_count = state.entries.len() - active_count;
            let is_all_completed = active_count == 0;

            html! {
                <div .todomvc-wrapper>
                    <section .todoapp>
                        <header .header>
                            <h1>"todos"</h1>
                            <EntryInput {link} />
                        </header>
                        <section .{main_class}>
                            {
                                html! {
                                    <input
                                        #toggle-all
                                        .toggle-all
                                        type="checkbox"
                                        checked={is_all_completed}
                                        onclick={link.callback(move |state, _| state.set_all(!is_all_completed))}
                                    />
                                }
                            }
                            <label for="toggle-all" />
                            <ul .todo-list>
                            {
                                state
                                    .filtered_entries()
                                    .map(move |(idx, entry)| html! { <EntryView {idx} {entry} {link} /> })
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
                            {
                                let selected = state.filter;

                                [
                                    Filter::All,
                                    Filter::Active,
                                    Filter::Completed,
                                ]
                                .map(|filter| html! { <FilterView {filter} {selected} {link} /> })
                            }
                            </ul>
                            <button .clear-completed onclick={link.callback(|state, _| state.entries.retain(|entry| !entry.completed))}>
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
    link: Link<'a, State>,
}

mod wat {
    use super::*;

    impl<'a> EntryInput<'a> {
        pub fn render(self) -> impl Html + 'a {
            let onchange = self.link.callback(|state, event| {
                let input: HtmlInputElement = event.target();

                let value = input.value();
                input.set_value("");

                state.add(value);

                ShouldRender::Yes
            });

            html! {
                <input .new-todo placeholder="What needs to be done?" {onchange} />
            }
        }
    }
}

struct EntryView<'a> {
    idx: usize,
    entry: &'a Entry,
    link: Link<'a, State>,
}

fn test(checked: bool) -> impl Html {
    html! { <p><input {checked} value="foo" width={ 42 } /></p> }
}

impl<'a> EntryView<'a> {
    fn render(self) -> impl Html + 'a {
        let EntryView { idx, entry, link } = self;

        let class = match (entry.editing, entry.completed) {
            (false, false) => "todo",
            (true, false) => "todo editing",
            (false, true) => "todo completed",
            (true, true) => "todo editing completed",
        };

        let input = self.entry.editing.then(move || {
            let onchange = link.callback(move |state, event| {
                let input: HtmlInputElement = event.target();

                state.entries[idx].update(input.value())
            });

            let onmouseover = link.callback(move |_, event| {
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
                    {onchange}
                />
            }
        });

        let onchange = link.callback(move |state, _| state.entries[idx].toggle());

        html! {
            <li {class}>
                <div .view>
                    {
                        html! {
                            <input .toggle type="checkbox" checked={entry.completed} {onchange} />
                        }
                    }
                    <label ondblclick={link.callback(move |state, _| state.edit_entry(idx))} >
                        { &entry.description }
                    </label>
                    <button .destroy onclick={link.callback(move |state, _| { state.entries.remove(idx); })} />
                </div>
                { input }
            </li>
        }
    }
}

struct FilterView<'a> {
    filter: Filter,
    selected: Filter,
    link: Link<'a, State>,
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
        let onclick = self.link.callback(move |state, _| state.filter = filter);

        html! {
            <li>
                <a {class} {href} {onclick}>{ filter.to_label() }</a>
            </li>
        }
    }
}

impl State {
    fn count_active(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.filter(Filter::Active))
            .count()
    }

    fn filtered_entries(&self) -> impl Iterator<Item = (usize, &Entry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.filter(self.filter))
    }

    fn set_all(&mut self, completed: bool) {
        for entry in self.entries.iter_mut() {
            entry.completed = completed;
        }
    }

    fn edit_entry(&mut self, idx: usize) {
        if let Some(entry) = self.editing.and_then(|idx| self.entries.get_mut(idx)) {
            entry.editing = false;
        }

        self.editing = Some(idx);
        self.entries[idx].editing = true;
    }

    fn add(&mut self, description: String) {
        self.entries.push(Entry {
            description,
            completed: false,
            editing: false,
        });
    }
}

fn main() {
    kobold::start(html! {
        <App />
    });
}
