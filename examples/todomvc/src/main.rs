use kobold::prelude::*;
use web_sys::HtmlInputElement as InputElement;

mod state;

use state::*;

fn app(state: &Hook<State>) -> impl View + '_ {
    let hidden = class!("hidden" if state.entries.is_empty());

    let active_count = state.count_active();
    let completed_hidden = class!("hidden" if state.entries.len() == active_count);

    view! {
        <div.todomvc-wrapper>
            <section.todoapp>
                <header.header>
                    <h1>"todos"</h1>
                    <!entry_input {state}>
                </header>
                <section .main.{hidden}>
                    <!toggle_all {active_count} {state}>
                    <ul.todo-list>
                        {
                            for state
                                .filtered_entries()
                                .map(move |(idx, entry)| view! { <!entry {idx} {entry} {state}> })
                        }
                </section>
                <footer.footer.{hidden}>
                    <span.todo-count>
                        <strong>{ active_count }</strong>
                        {
                            ref match active_count {
                                1 => " item left",
                                _ => " items left",
                            }
                        }
                    </span>
                    <ul.filters>
                        <!filter by={Filter::All} {state}>
                        <!filter by={Filter::Active} {state}>
                        <!filter by={Filter::Completed} {state}>
                    </ul>
                    <button.clear-completed.{completed_hidden} onclick={do state.clear()}> "Clear completed"
            </section>
            <footer.info>
                <p> "Double-click to edit a todo"
                <p> "Written by "<a href="https://maciej.codes/" target="_blank">"Maciej Hirsz"</a>
                <p> "Part of "<a href="http://todomvc.com/" target="_blank">"TodoMVC"</a>
    }
}

#[component]
fn entry_input(state: &Hook<State>) -> impl View + '_ {
    let onchange = event!(|state, e: Event<InputElement>| {
        let input = e.target();
        state.add(input.value());

        input.set_value("");
    });

    view! {
        <input.new-todo placeholder="What needs to be done?" {onchange}>
    }
}

#[component]
fn toggle_all(active_count: usize, state: &Hook<State>) -> impl View + '_ {
    view! {
        <input #toggle-all.toggle-all
            type="checkbox"
            checked={active_count == 0}
            onclick={do state.set_all(active_count != 0)}
        >
        <label for="toggle-all">
    }
}

#[component]
fn entry<'a>(idx: usize, entry: &'a Entry, state: &'a Hook<State>) -> impl View + 'a {
    let input = entry.editing.then(move || {
        let onkeypress = event!(move |state, e: KeyboardEvent<InputElement>| {
            if e.key() == "Enter" {
                state.update(idx, e.target().value());

                Then::Render
            } else {
                Then::Stop
            }
        });
        let onblur = event!(move |state, e: Event<InputElement>| {
            state.update(idx, e.target().value());
        });

        view! {
            <input.edit
                type="text"
                value={static &entry.description}
                onmouseover={|event| event.target().focus()}
                {onkeypress}
                {onblur}
            >
        }
    });

    let editing = class!("editing" if entry.editing);
    let completed = class!("completed" if entry.completed);

    view! {
        <li.todo.{editing}.{completed}>
            <div.view>
                <input.toggle type="checkbox" checked={entry.completed} onchange={do state.toggle(idx)}>
                <label ondblclick={do state.edit_entry(idx)} >
                    { ref entry.description }
                </label>
                <button.destroy onclick={do state.remove(idx)}>
            </div>
            { input }
    }
}

#[component]
fn filter(by: Filter, state: &Hook<State>) -> impl View + '_ {
    let class = class!("selected" if state.filter == by);

    view! {
        <li>
            <a {class} onclick={do state.filter = by} href={static by.href()}>
                { static by.label() }
    }
}

fn main() {
    kobold::start(stateful(State::default, app));
}
