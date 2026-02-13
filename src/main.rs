use leptos::prelude::*;
use rand::thread_rng;
use wasm_bindgen::prelude::*;

use eu_test_data_generator::{iban, personal_id};

fn main() {
    leptos::mount::mount_to_body(App);
}

#[derive(Clone, Debug)]
struct IbanRow {
    raw: String,
    formatted: String,
    valid: bool,
}

#[derive(Clone, Debug)]
struct IdRow {
    code: String,
    gender: String,
    dob: String,
    valid: bool,
}

#[wasm_bindgen(inline_js = r#"
export function copy_text(text) {
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text).catch(() => {});
    }
}
"#)]
extern "C" {
    fn copy_text(text: &str);
}

fn copy_to_clipboard(text: &str) {
    copy_text(text);
}

#[component]
fn App() -> impl IntoView {
    let active_tab = RwSignal::new("iban");

    view! {
        <div class="app">
            <header>
                <h1>"MockBanker"</h1>
                <p>"Generate valid, checksum-correct IBANs and personal IDs \u{2014} runs entirely in your browser"</p>
            </header>

            <div class="tabs">
                <button
                    class=move || if active_tab.get() == "iban" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("iban")
                >
                    "IBAN"
                </button>
                <button
                    class=move || if active_tab.get() == "id" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("id")
                >
                    "Personal ID"
                </button>
            </div>

            <Show when=move || active_tab.get() == "iban">
                <IbanTab />
            </Show>
            <Show when=move || active_tab.get() == "id">
                <PersonalIdTab />
            </Show>

            <footer>
                <p>
                    "Powered by "
                    <a href="https://github.com/Sunyata-OU/EU-Test-Data-Generator" target="_blank">"eu-test-data-generator"</a>
                    " \u{00b7} Built with Rust & WebAssembly"
                </p>
            </footer>
        </div>
    }
}

#[component]
fn IbanTab() -> impl IntoView {
    let countries = iban::supported_countries();
    let country = RwSignal::new("DE".to_string());
    let count = RwSignal::new(5u32);
    let spaces = RwSignal::new(true);
    let results: RwSignal<Vec<IbanRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let c_opt = if c == "Random" {
            None
        } else {
            Some(c.as_str())
        };
        let mut rows = Vec::new();
        for _ in 0..n {
            if let Ok(code) = iban::generate_iban(c_opt, &mut rng) {
                let valid = iban::validate_iban(&code);
                rows.push(IbanRow {
                    formatted: iban::format_iban(&code),
                    raw: code,
                    valid,
                });
            }
        }
        results.set(rows);
        copied_idx.set(None);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let use_spaces = spaces.get();
        let text: String = rows
            .iter()
            .map(|r| {
                if use_spaces {
                    r.formatted.as_str()
                } else {
                    r.raw.as_str()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        copy_to_clipboard(&text);
    };

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <select on:change=move |ev| {
                    country.set(event_target_value(&ev));
                }>
                    <option value="Random">"Random"</option>
                    {countries.into_iter().map(|cc| {
                        let cc_owned = cc.to_string();
                        let cc2 = cc_owned.clone();
                        view! {
                            <option value={cc_owned} selected=move || country.get() == cc2>
                                {cc}
                            </option>
                        }
                    }).collect_view()}
                </select>
            </div>

            <div class="field">
                <label>"Count"</label>
                <input type="number" min="1" max="100"
                    prop:value=move || count.get().to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                            count.set(v.clamp(1, 100));
                        }
                    }
                />
            </div>

            <div class="checkbox-field">
                <input type="checkbox" id="spaces"
                    prop:checked=move || spaces.get()
                    on:change=move |_| spaces.update(|s| *s = !*s)
                />
                <label for="spaces">"Spaces"</label>
            </div>

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
            </Show>
        </div>

        <Show when=move || results.get().is_empty()>
            <div class="empty">"Select a country and click Generate"</div>
        </Show>

        <Show when=move || !results.get().is_empty()>
            <div class="results-header">
                <span>{move || format!("{} results", results.get().len())}</span>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>"IBAN"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let use_spaces = spaces.get();
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let display = if use_spaces { row.formatted.clone() } else { row.raw.clone() };
                            let copy_text = display.clone();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{display}</td>
                                    <td class={valid_class}>{if row.valid { "Yes" } else { "No" }}</td>
                                    <td>
                                        <button
                                            class=if is_copied { "btn-copy copied" } else { "btn-copy" }
                                            on:click=move |_| {
                                                copy_to_clipboard(&copy_text);
                                                copied_idx.set(Some(i));
                                            }
                                        >
                                            {if is_copied { "Copied!" } else { "Copy" }}
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
            </table>
        </Show>
    }
}

#[component]
fn PersonalIdTab() -> impl IntoView {
    let registry = personal_id::Registry::new();
    let id_countries: Vec<(String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n)| (c.to_string(), n.to_string()))
        .collect();

    let country = RwSignal::new("EE".to_string());
    let count = RwSignal::new(5u32);
    let gender = RwSignal::new("any".to_string());
    let year = RwSignal::new(String::new());
    let results: RwSignal<Vec<IdRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let g = gender.get();
        let y: Option<u16> = year.get().parse().ok();
        let gender_opt = match g.as_str() {
            "male" => Some(personal_id::date::Gender::Male),
            "female" => Some(personal_id::date::Gender::Female),
            _ => None,
        };
        let opts = personal_id::GenOptions {
            gender: gender_opt,
            year: y,
        };
        let mut rows = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                if let Some(code) = reg.generate(&c, &opts, &mut rng)
                    && let Some(parsed) = reg.parse(&c, &code)
                {
                    rows.push(IdRow {
                        code: parsed.code,
                        gender: parsed.gender.unwrap_or_default(),
                        dob: parsed.dob.unwrap_or_default(),
                        valid: parsed.valid,
                    });
                }
            }
        });
        results.set(rows);
        copied_idx.set(None);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let text: String = rows
            .iter()
            .map(|r| r.code.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        copy_to_clipboard(&text);
    };

    let countries_for_select = id_countries.clone();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <select on:change=move |ev| {
                    country.set(event_target_value(&ev));
                }>
                    {countries_for_select.into_iter().map(|(code, name)| {
                        let code2 = code.clone();
                        let label = format!("{code} \u{2014} {name}");
                        view! {
                            <option value={code} selected=move || country.get() == code2>
                                {label}
                            </option>
                        }
                    }).collect_view()}
                </select>
            </div>

            <div class="field">
                <label>"Count"</label>
                <input type="number" min="1" max="100"
                    prop:value=move || count.get().to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                            count.set(v.clamp(1, 100));
                        }
                    }
                />
            </div>

            <div class="field">
                <label>"Gender"</label>
                <select on:change=move |ev| {
                    gender.set(event_target_value(&ev));
                }>
                    <option value="any">"Any"</option>
                    <option value="male">"Male"</option>
                    <option value="female">"Female"</option>
                </select>
            </div>

            <div class="field">
                <label>"Year"</label>
                <input type="text" placeholder="any"
                    prop:value=move || year.get()
                    on:input=move |ev| {
                        year.set(event_target_value(&ev));
                    }
                />
            </div>

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
            </Show>
        </div>

        <Show when=move || results.get().is_empty()>
            <div class="empty">"Select a country and click Generate"</div>
        </Show>

        <Show when=move || !results.get().is_empty()>
            <div class="results-header">
                <span>{move || format!("{} results", results.get().len())}</span>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>"Code"</th>
                        <th>"Gender"</th>
                        <th>"Date of Birth"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_code = code.clone();
                            let gender_str = row.gender.clone();
                            let dob = row.dob.clone();
                            let valid = row.valid;
                            let valid_class = if valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td class="gender">{gender_str}</td>
                                    <td class="dob">{dob}</td>
                                    <td class={valid_class}>{if valid { "Yes" } else { "No" }}</td>
                                    <td>
                                        <button
                                            class=if is_copied { "btn-copy copied" } else { "btn-copy" }
                                            on:click=move |_| {
                                                copy_to_clipboard(&copy_code);
                                                copied_idx.set(Some(i));
                                            }
                                        >
                                            {if is_copied { "Copied!" } else { "Copy" }}
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
            </table>
        </Show>
    }
}
