use kobold::prelude::*;
use kobold::reexport::wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

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

    fn update(&mut self, event: &Event) -> ShouldRender {
        let value = event
            .target()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .value();

        self.editing = false;
        self.description = value;

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
}

impl App {
    fn render(self) -> impl Html {
        self.stateful(|state, link| {
            let (main_class, footer_class) = if state.entries.is_empty() {
                ("main hidden", "main hidden")
            } else {
                ("main", "footer")
            };

            let selected = state.filter;
            let active_count = state.count_active();
            let completed_count = state.entries.len() - active_count;
            let is_all_completed = active_count == 0;

            html! {
                <div class="todomvc-wrapper">
                    <section class="todoapp">
                        <header class="header">
                            <h1>"todos"</h1>
                            <EntryInput {link} />
                        </header>
                        <section class={main_class}>
                            {
                                html! {
                                    <input
                                        type="checkbox"
                                        class="toggle-all"
                                        id="toggle-all"
                                        checked={is_all_completed}
                                        onclick={link.callback(move |state, _| state.set_all(!is_all_completed))}
                                    />
                                }
                            }
                            <label for="toggle-all" />
                            <ul class="todo-list">
                            {
                                state
                                    .filtered_entries()
                                    .map(move |(idx, entry)| html! { <EntryView {idx} {entry} {link} /> })
                                    .list()
                            }
                            </ul>
                        </section>
                        <footer class={footer_class}>
                            <span class="todo-count">
                                <strong>{ active_count }</strong>
                                { if active_count == 1 { " item left" } else { " items left" } }
                            </span>
                            <ul class="filters">
                                <FilterView filter={Filter::All} {selected} {link}>"All"</FilterView>
                                <FilterView filter={Filter::Active} {selected} {link}>"Active"</FilterView>
                                <FilterView filter={Filter::Completed} {selected} {link}>"Completed"</FilterView>
                            </ul>
                            <button class="clear-completed" onclick={link.callback(|state, _| state.entries.retain(|entry| !entry.completed))}>
                                "Clear completed ("{ completed_count }")"
                            </button>
                        </footer>
                    </section>
                    <footer class="info">
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

impl<'a> EntryInput<'a> {
    fn render(self) -> impl Html + 'a {
        let onchange = self.link.callback(|state, e| {
            let input: HtmlInputElement = e.target().unwrap().unchecked_into();

            let value = input.value();
            input.set_value("");

            state.add(value);
        });

        html! {
            <input class="new-todo" placeholder="What needs to be done?" {onchange} />
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
            let onchange = link.callback(move |state, event| state.entries[idx].update(event));

            let onmouseover = link.callback(move |_, event| {
                let input = event
                    .target()
                    .unwrap()
                    .unchecked_into::<HtmlInputElement>();

                input.focus().unwrap();
                input.select();

                ShouldRender::No
            });

            html! {
                <input
                    class="edit"
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
                <div class="view">
                    {
                        html! {
                            <input type="checkbox" class="toggle" checked={entry.completed} {onchange} />
                        }
                    }
                    <label ondblclick={link.callback(move |state, _| state.edit_entry(idx))} >
                        { &entry.description }
                    </label>
                    <button class="destroy" onclick={link.callback(move |state, _| { state.entries.remove(idx); })} />
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
    fn render_with(self, name: impl Html + 'a) -> impl Html + 'a {
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
                <a {class} {href} {onclick}>{ name }</a>
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
