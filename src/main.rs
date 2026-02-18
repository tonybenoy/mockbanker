use leptos::prelude::*;
use leptos::task::spawn_local;
use rand::thread_rng;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use idsmith::{bank_account, company_id, countries, credit_card, driver_license, iban, lei, passport, personal_id, swift, tax_id, vat};

#[wasm_bindgen(inline_js = r#"
export function copy_text(text) {
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text).catch(() => {});
    }
}

export function toggle_theme() {
    const root = document.documentElement;
    const current = root.getAttribute("data-theme");
    let next;
    if (current === "light") {
        next = "dark";
    } else if (current === "dark") {
        next = "light";
    } else {
        next = window.matchMedia("(prefers-color-scheme: light)").matches ? "dark" : "light";
    }
    root.setAttribute("data-theme", next);
    localStorage.setItem("theme", next);
    return next === "light";
}

export function init_theme() {
    const saved = localStorage.getItem("theme");
    if (saved) {
        document.documentElement.setAttribute("data-theme", saved);
        return saved === "light";
    }
    return window.matchMedia("(prefers-color-scheme: light)").matches;
}

export function download_file(filename, content, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

export function check_online(callback) {
    window.addEventListener('online', () => callback(true));
    window.addEventListener('offline', () => callback(false));
    return navigator.onLine;
}

export function register_pwa_install(callback) {
    window.addEventListener('beforeinstallprompt', (e) => {
        e.preventDefault();
        window.deferredPrompt = e;
        callback(true);
    });
}

export async function trigger_pwa_install() {
    const promptEvent = window.deferredPrompt;
    if (!promptEvent) return;
    promptEvent.prompt();
    const { outcome } = await promptEvent.userChoice;
    window.deferredPrompt = null;
    return outcome === 'accepted';
}
"#)]
extern "C" {
    fn copy_text(text: &str);
    fn toggle_theme() -> bool;
    fn init_theme() -> bool;
    fn download_file(filename: &str, content: &str, mimeType: &str);
    fn check_online(callback: js_sys::Function) -> bool;
    fn register_pwa_install(callback: js_sys::Function);
    fn trigger_pwa_install() -> js_sys::Promise;
}

fn copy_to_clipboard(text: &str) {
    copy_text(text);
}

fn download_csv(filename: &str, content: &str) {
    download_file(filename, content, "text/csv;charset=utf-8;");
}

fn country_name(code: &str) -> &'static str {
    countries::get_country_name(code).unwrap_or("Unknown")
}

fn main() {
    leptos::mount::mount_to_body(App);
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct IbanRow {
    raw: String,
    formatted: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct IdRow {
    code: String,
    gender: String,
    dob: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BankAccountRow {
    account: String,
    routing: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CreditCardRow {
    number: String,
    brand: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SwiftRow {
    code: String,
    bank: String,
    country: String,
    location: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CompanyIdRow {
    code: String,
    name: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DriverLicenseRow {
    code: String,
    name: String,
    country: String,
    state: Option<String>,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PassportRow {
    code: String,
    name: String,
    country: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct TaxIdRow {
    code: String,
    name: String,
    country: String,
    holder_type: Option<String>,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct VatRow {
    code: String,
    country_code: String,
    country_name: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LeiRow {
    code: String,
    lou: String,
    country_code: String,
    valid: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct HistoryItem {
    id: String,
    timestamp: u64,
    category: String,
    country: String,
    count: u32,
    results: Vec<String>,
}

#[component]
fn Tooltip(text: String) -> impl IntoView {
    view! {
        <div class="tooltip-container">
            <span class="tooltip-icon">"?"</span>
            <div class="tooltip-content">{text}</div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let active_tab = RwSignal::new("iban");
    let is_light = RwSignal::new(init_theme());

    let is_online = RwSignal::new(true);
    let online_cb = Closure::wrap(Box::new(move |online: bool| {
        is_online.set(online);
    }) as Box<dyn FnMut(bool)>);
    is_online.set(check_online(online_cb.into_js_value().unchecked_into()));

    let can_install = RwSignal::new(false);
    let install_cb = Closure::wrap(Box::new(move |can: bool| {
        can_install.set(can);
    }) as Box<dyn FnMut(bool)>);
    register_pwa_install(install_cb.into_js_value().unchecked_into());

    let install_app = move |_| {
        spawn_local(async move {
            let res = wasm_bindgen_futures::JsFuture::from(trigger_pwa_install()).await;
            if let Ok(val) = res
                && val.as_bool().unwrap_or(false)
            {
                can_install.set(false);
            }
        });
    };

    view! {
        <div class="app">
            <header>
                <div class="header-main">
                    <img src="assets/logo.svg" alt="MockBanker Logo" class="logo" />
                    <h1>"MockBanker"</h1>
                    <div class="header-badges">
                        <Show when=move || !is_online.get()>
                            <span class="badge badge-offline">"Offline"</span>
                        </Show>
                        <Show when=move || can_install.get()>
                            <button class="btn-install" on:click=install_app>"Install App"</button>
                        </Show>
                    </div>
                </div>
                <p>"Generate valid, checksum-correct test data \u{2014} runs entirely in your browser"</p>
                <button
                    class="theme-toggle"
                    aria-label="Toggle theme"
                    on:click=move |_| { is_light.set(toggle_theme()); }
                >
                    {move || if is_light.get() { "\u{263e}" } else { "\u{2600}" }}
                </button>
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
                <button
                    class=move || if active_tab.get() == "bank" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("bank")
                >
                    "Bank Account"
                </button>
                <button
                    class=move || if active_tab.get() == "card" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("card")
                >
                    "Credit Card"
                </button>
                <button
                    class=move || if active_tab.get() == "swift" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("swift")
                >
                    "SWIFT/BIC"
                </button>
                <button
                    class=move || if active_tab.get() == "company" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("company")
                >
                    "Company ID"
                </button>
                <button
                    class=move || if active_tab.get() == "driver_license" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("driver_license")
                >
                    "Driver's License"
                </button>
                <button
                    class=move || if active_tab.get() == "passport" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("passport")
                >
                    "Passport"
                </button>
                <button
                    class=move || if active_tab.get() == "tax_id" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("tax_id")
                >
                    "Tax ID"
                </button>
                <button
                    class=move || if active_tab.get() == "vat" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("vat")
                >
                    "VAT"
                </button>
                <button
                    class=move || if active_tab.get() == "lei" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("lei")
                >
                    "LEI"
                </button>
                <button
                    class=move || if active_tab.get() == "validator" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("validator")
                >
                    "Validator"
                </button>
                <button
                    class=move || if active_tab.get() == "history" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("history")
                >
                    "History"
                </button>
            </div>

            <Show when=move || active_tab.get() == "iban">
                <IbanTab />
            </Show>
            <Show when=move || active_tab.get() == "id">
                <PersonalIdTab />
            </Show>
            <Show when=move || active_tab.get() == "bank">
                <BankAccountTab />
            </Show>
            <Show when=move || active_tab.get() == "card">
                <CreditCardTab />
            </Show>
            <Show when=move || active_tab.get() == "swift">
                <SwiftTab />
            </Show>
            <Show when=move || active_tab.get() == "company">
                <CompanyIdTab />
            </Show>
            <Show when=move || active_tab.get() == "driver_license">
                <DriverLicenseTab />
            </Show>
            <Show when=move || active_tab.get() == "passport">
                <PassportTab />
            </Show>
            <Show when=move || active_tab.get() == "tax_id">
                <TaxIdTab />
            </Show>
            <Show when=move || active_tab.get() == "vat">
                <VatTab />
            </Show>
            <Show when=move || active_tab.get() == "lei">
                <LeiTab />
            </Show>
            <Show when=move || active_tab.get() == "validator">
                <ValidatorTab />
            </Show>
            <Show when=move || active_tab.get() == "history">
                <HistoryTab />
            </Show>

            <footer>
                <p>
                    "Built with \u{2764} by "
                    <a href="https://tonybenoy.com" target="_blank">"Tony Benoy"</a>
                    " \u{00b7} "
                    <a href="https://github.com/tonybenoy/mockbanker" target="_blank">"GitHub"</a>
                    " \u{00b7} "
                    <a href="https://github.com/tonybenoy/mockbanker/issues" target="_blank">"Contribute"</a>
                </p>
                <div class="share-links">
                    <span>"Share: "</span>
                    <a href="https://www.facebook.com/sharer/sharer.php?u=https://tonybenoy.github.io/mockbanker/" target="_blank">"Facebook"</a>
                    <a href="https://twitter.com/intent/tweet?url=https://tonybenoy.github.io/mockbanker/&text=MockBanker%20%E2%80%94%20Free%20IBAN%20%26%20Personal%20ID%20Generator" target="_blank">"Twitter"</a>
                    <a href="https://www.linkedin.com/sharing/share-offsite/?url=https://tonybenoy.github.io/mockbanker/" target="_blank">"LinkedIn"</a>
                </div>
                <p style="margin-top: 0.5rem; opacity: 0.8;">
                    "Powered by "
                    <a href="https://github.com/Sunyata-OU/idsmith" target="_blank">"idsmith"</a>
                    " \u{00b7} Built with "
                    <a href="https://gemini.google.com" target="_blank">"Gemini"</a>
                    " & "
                    <a href="https://claude.ai" target="_blank">"Claude"</a>
                </p>
            </footer>
        </div>
    }
}

fn add_to_history(category: &str, country: &str, count: u32, results: Vec<String>) {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    let mut history: Vec<HistoryItem> = storage
        .get_item("history")
        .unwrap()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let item = HistoryItem {
        id: rand::random::<u64>().to_string(),
        timestamp: js_sys::Date::now() as u64,
        category: category.to_string(),
        country: country.to_string(),
        count,
        results,
    };

    history.insert(0, item);
    if history.len() > 50 {
        history.truncate(50);
    }

    if let Ok(json) = serde_json::to_string(&history) {
        let _ = storage.set_item("history", &json);
    }
}

#[component]
fn HistoryTab() -> impl IntoView {
    let get_history = || {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        storage
            .get_item("history")
            .unwrap()
            .and_then(|s| serde_json::from_str::<Vec<HistoryItem>>(&s).ok())
            .unwrap_or_default()
    };

    let history = RwSignal::new(get_history());

    let clear_history = move |_| {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        let _ = storage.remove_item("history");
        history.set(Vec::new());
    };

    view! {
        <div class="history-tab">
            <div class="controls">
                <button class="btn btn-secondary" on:click=clear_history>"Clear History"</button>
            </div>

            <Show when=move || history.get().is_empty()>
                <div class="empty">"No history yet. Generate some data to see it here!"</div>
            </Show>

            <div class="history-list">
                {move || history.get().into_iter().map(|item| {
                    let date = js_sys::Date::new(&js_sys::Number::from(item.timestamp as f64));
                    let date_str = format!("{}/{}/{} {}:{:02}",
                        date.get_date(), date.get_month() + 1, date.get_full_year(),
                        date.get_hours(), date.get_minutes());

                    view! {
                        <div class="history-item">
                            <div class="history-meta">
                                <span class="history-category">{item.category}</span>
                                <span class="history-country">{item.country}</span>
                                <span class="history-count">{item.count} " items"</span>
                                <span class="history-date">{date_str}</span>
                            </div>
                            <div class="history-results">
                                {item.results.join(", ")}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn IbanTab() -> impl IntoView {
    let mut countries: Vec<&str> = iban::supported_countries();
    countries.sort_by_key(|c| country_name(c));
    let country = RwSignal::new("DE".to_string());
    let count = RwSignal::new(5u32);
    let spaces = RwSignal::new(true);
    let results: RwSignal<Vec<IbanRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let countries_list: Vec<(String, String)> = countries
        .into_iter()
        .map(|c| (c.to_string(), country_name(c).to_string()))
        .collect();

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
        let mut history_results = Vec::new();
        for _ in 0..n {
            if let Ok(code) = iban::generate_iban(c_opt, &mut rng) {
                let valid = iban::validate_iban(&code);
                rows.push(IbanRow {
                    formatted: iban::format_iban(&code),
                    raw: code.clone(),
                    valid,
                });
                history_results.push(code);
            }
        }
        results.set(rows);
        copied_idx.set(None);
        add_to_history("IBAN", &c, n, history_results);
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

    let save_csv = move |_| {
        let rows = results.get();
        let use_spaces = spaces.get();
        let mut csv = String::from("IBAN,Valid\n");
        for row in rows.iter() {
            let display = if use_spaces { &row.formatted } else { &row.raw };
            csv.push_str(&format!(
                "{},{}\n",
                display,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_file("ibans.csv", &csv, "text/csv;charset=utf-8;");
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("ibans.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql =
            String::from("CREATE TABLE IF NOT EXISTS ibans (iban TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO ibans (iban, valid) VALUES ('{}', {});\n",
                row.raw, row.valid
            ));
        }
        download_file("ibans.sql", &sql, "text/plain;charset=utf-8;");
    };

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_list
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
    let id_countries: Vec<(String, String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n, d)| (c.to_string(), n.to_string(), d.to_string()))
        .collect();

    let country = RwSignal::new("EE".to_string());
    let count = RwSignal::new(5u32);
    let gender = RwSignal::new("any".to_string());
    let year = RwSignal::new(String::new());
    let results: RwSignal<Vec<IdRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);
    let id_countries_stored = StoredValue::new(id_countries.clone());

    let current_description = Memo::new(move |_| {
        let c = country.get();
        id_countries_stored.with_value(|list| {
            list.iter()
                .find(|(code, _, _)| code == &c)
                .map(|(_, _, d)| d.clone())
                .unwrap_or_default()
        })
    });

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
        let mut history_results = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                if let Some(code) = reg.generate(&c, &opts, &mut rng)
                    && let Some(parsed) = reg.parse(&c, &code)
                {
                    rows.push(IdRow {
                        code: parsed.code.clone(),
                        gender: parsed.gender.unwrap_or_default(),
                        dob: parsed.dob.unwrap_or_default(),
                        valid: parsed.valid,
                    });
                    history_results.push(parsed.code);
                }
            }
        });
        results.set(rows);
        copied_idx.set(None);
        add_to_history("Personal ID", &c, n, history_results);
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

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,Gender,Date of Birth,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                row.code,
                row.gender,
                row.dob,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_file("personal_ids.csv", &csv, "text/csv;charset=utf-8;");
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file(
            "personal_ids.json",
            &json,
            "application/json;charset=utf-8;",
        );
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from(
            "CREATE TABLE IF NOT EXISTS personal_ids (code TEXT, gender TEXT, dob TEXT, valid BOOLEAN);\n",
        );
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO personal_ids (code, gender, dob, valid) VALUES ('{}', '{}', '{}', {});\n",
                row.code, row.gender, row.dob, row.valid
            ));
        }
        download_file("personal_ids.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = id_countries
        .clone()
        .into_iter()
        .map(|(c, n, _)| (c, n))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>
                    "Country "
                    <Tooltip text=current_description.get() />
                </label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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

#[component]
fn BankAccountTab() -> impl IntoView {
    let registry = bank_account::Registry::new();
    let countries: Vec<(String, String, String, bool)> = registry
        .list_countries()
        .iter()
        .map(|(c, n, d, i)| (c.to_string(), n.to_string(), d.to_string(), *i))
        .collect();

    let country = RwSignal::new("US".to_string());
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<BankAccountRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let mut rows = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = bank_account::GenOptions::default();
                if let Some(res) = reg.generate(&c, &opts, &mut rng) {
                    rows.push(BankAccountRow {
                        account: res.account_number,
                        routing: res.bank_code.unwrap_or_default(),
                        valid: res.valid,
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
            .map(|r| {
                if r.routing.is_empty() {
                    r.account.clone()
                } else {
                    format!("{} ({})", r.account, r.routing)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Account,Routing,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{}\n",
                row.account,
                row.routing,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("bank_accounts.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("bank_accounts.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS bank_accounts (account TEXT, routing TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO bank_accounts (account, routing, valid) VALUES ('{}', '{}', {});\n",
                row.account, row.routing, row.valid
            ));
        }
        download_file("bank_accounts.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = countries
        .clone()
        .into_iter()
        .map(|(c, n, _, _)| (c, n))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"Account"</th>
                        <th>"Routing"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let account = row.account.clone();
                            let routing = row.routing.clone();
                            let copy_text = if routing.is_empty() { account.clone() } else { format!("{} ({})", account, routing) };
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{account}</td>
                                    <td>{routing}</td>
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
fn CreditCardTab() -> impl IntoView {
    let registry = credit_card::Registry::new();
    let brands: Vec<String> = registry
        .list_brands()
        .iter()
        .map(|b| b.to_string())
        .collect();

    let brand = RwSignal::new("visa".to_string());
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<CreditCardRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let b = brand.get();
        let n = count.get();
        let mut rows = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = credit_card::GenOptions {
                    brand: Some(b.clone()),
                };
                if let Some(res) = reg.generate(&opts, &mut rng) {
                    rows.push(CreditCardRow {
                        number: res.number,
                        brand: res.brand,
                        valid: res.valid,
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
            .map(|r| r.number.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Number,Brand,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{}\n",
                row.number,
                row.brand,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("credit_cards.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("credit_cards.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS credit_cards (number TEXT, brand TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO credit_cards (number, brand, valid) VALUES ('{}', '{}', {});\n",
                row.number, row.brand, row.valid
            ));
        }
        download_file("credit_cards.sql", &sql, "text/plain;charset=utf-8;");
    };

    let brands_for_select = brands.clone();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Brand"</label>
                <select on:change=move |ev| {
                    brand.set(event_target_value(&ev));
                }>
                    {brands_for_select.into_iter().map(|id| {
                        let id2 = id.clone();
                        let label = id.clone();
                        view! {
                            <option value={id} selected=move || brand.get() == id2>
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
            </Show>
        </div>

        <Show when=move || results.get().is_empty()>
            <div class="empty">"Select a brand and click Generate"</div>
        </Show>

        <Show when=move || !results.get().is_empty()>
            <div class="results-header">
                <span>{move || format!("{} results", results.get().len())}</span>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>"Number"</th>
                        <th>"Brand"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let number = row.number.clone();
                            let copy_text = number.clone();
                            let brand = row.brand.clone();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{number}</td>
                                    <td>{brand}</td>
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
fn SwiftTab() -> impl IntoView {
    let registry = swift::Registry::new();
    let countries: Vec<String> = iban::supported_countries()
        .into_iter()
        .map(|c| c.to_string())
        .collect();

    let country = RwSignal::new("DE".to_string());
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<SwiftRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let mut rows = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = swift::GenOptions {
                    country: Some(c.clone()),
                };
                let res = reg.generate(&opts, &mut rng);
                rows.push(SwiftRow {
                    code: res.code,
                    bank: res.bank,
                    country: res.country,
                    location: res.location,
                    valid: res.valid,
                });
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

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("SWIFT/BIC,Bank,Country,Location,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                row.code,
                row.bank,
                row.country,
                row.location,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("swift_codes.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("swift_codes.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS swift_codes (code TEXT, bank TEXT, country TEXT, location TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO swift_codes (code, bank, country, location, valid) VALUES ('{}', '{}', '{}', '{}', {});\n",
                row.code, row.bank, row.country, row.location, row.valid
            ));
        }
        download_file("swift_codes.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = countries
        .clone()
        .into_iter()
        .map(|code| (code.clone(), country_name(&code).to_string()))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"SWIFT/BIC"</th>
                        <th>"Bank"</th>
                        <th>"Country"</th>
                        <th>"Location"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let bank = row.bank.clone();
                            let country = row.country.clone();
                            let location = row.location.clone();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{bank}</td>
                                    <td>{country}</td>
                                    <td>{location}</td>
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
fn CompanyIdTab() -> impl IntoView {
    let registry = company_id::Registry::new();
    let countries: Vec<(String, String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n, d)| (c.to_string(), n.to_string(), d.to_string()))
        .collect();

    let country = RwSignal::new("EE".to_string());
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<CompanyIdRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let mut rows = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = company_id::GenOptions {
                    country: Some(c.clone()),
                };
                if let Some(res) = reg.generate(&opts, &mut rng) {
                    rows.push(CompanyIdRow {
                        code: res.code,
                        name: res.name,
                        valid: res.valid,
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

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,Name,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{}\n",
                row.code,
                row.name,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("company_ids.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("company_ids.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS company_ids (code TEXT, name TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO company_ids (code, name, valid) VALUES ('{}', '{}', {});\n",
                row.code, row.name, row.valid
            ));
        }
        download_file("company_ids.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = countries
        .clone()
        .into_iter()
        .map(|(c, n, _)| (c, n))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"Name"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let name = row.name.clone();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{name}</td>
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
fn DriverLicenseTab() -> impl IntoView {
    let registry = driver_license::Registry::new();
    let countries: Vec<(String, String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n, d)| (c.to_string(), n.to_string(), d.to_string()))
        .collect();

    let country = RwSignal::new(
        countries
            .first()
            .map(|(c, _, _)| c.clone())
            .unwrap_or_default(),
    );
    let count = RwSignal::new(5u32);
    let state_input = RwSignal::new(String::new());
    let results: RwSignal<Vec<DriverLicenseRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let s = state_input.get();
        let mut rows = Vec::new();
        let mut history_results = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = driver_license::GenOptions {
                    country: Some(c.clone()),
                    state: if s.is_empty() { None } else { Some(s.clone()) },
                };
                if let Some(res) = reg.generate(&opts, &mut rng) {
                    history_results.push(res.code.clone());
                    rows.push(DriverLicenseRow {
                        code: res.code,
                        name: res.name,
                        country: format!("{} — {}", res.country_code, res.country_name),
                        state: res.state,
                        valid: res.valid,
                    });
                }
            }
        });
        results.set(rows);
        copied_idx.set(None);
        add_to_history("Driver's License", &c, n, history_results);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let text: String = rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,Name,Country,State,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                row.code,
                row.name,
                row.country,
                row.state.as_deref().unwrap_or(""),
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("driver_licenses.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("driver_licenses.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS driver_licenses (code TEXT, name TEXT, country TEXT, state TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO driver_licenses (code, name, country, state, valid) VALUES ('{}', '{}', '{}', '{}', {});\n",
                row.code, row.name, row.country, row.state.as_deref().unwrap_or(""), row.valid
            ));
        }
        download_file("driver_licenses.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = countries
        .into_iter()
        .map(|(c, n, _)| (c, n))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
            </div>

            <div class="field">
                <label>"State (optional)"</label>
                <input type="text" placeholder="e.g. CA"
                    prop:value=move || state_input.get()
                    on:input=move |ev| state_input.set(event_target_value(&ev))
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"Name"</th>
                        <th>"State"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let name = row.name.clone();
                            let state = row.state.clone().unwrap_or_default();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{name}</td>
                                    <td>{state}</td>
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
fn PassportTab() -> impl IntoView {
    let registry = passport::Registry::new();
    let countries: Vec<(String, String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n, d)| (c.to_string(), n.to_string(), d.to_string()))
        .collect();

    let country = RwSignal::new(
        countries
            .first()
            .map(|(c, _, _)| c.clone())
            .unwrap_or_default(),
    );
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<PassportRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let mut rows = Vec::new();
        let mut history_results = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = passport::GenOptions {
                    country: Some(c.clone()),
                };
                if let Some(res) = reg.generate(&opts, &mut rng) {
                    history_results.push(res.code.clone());
                    rows.push(PassportRow {
                        code: res.code,
                        name: res.name,
                        country: format!("{} — {}", res.country_code, res.country_name),
                        valid: res.valid,
                    });
                }
            }
        });
        results.set(rows);
        copied_idx.set(None);
        add_to_history("Passport", &c, n, history_results);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let text: String = rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,Name,Country,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                row.code,
                row.name,
                row.country,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("passports.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("passports.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS passports (code TEXT, name TEXT, country TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO passports (code, name, country, valid) VALUES ('{}', '{}', '{}', {});\n",
                row.code, row.name, row.country, row.valid
            ));
        }
        download_file("passports.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = countries
        .into_iter()
        .map(|(c, n, _)| (c, n))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"Name"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let name = row.name.clone();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{name}</td>
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
fn TaxIdTab() -> impl IntoView {
    let registry = tax_id::Registry::new();
    let countries: Vec<(String, String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n, d)| (c.to_string(), n.to_string(), d.to_string()))
        .collect();

    let country = RwSignal::new(
        countries
            .first()
            .map(|(c, _, _)| c.clone())
            .unwrap_or_default(),
    );
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<TaxIdRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let mut rows = Vec::new();
        let mut history_results = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = tax_id::GenOptions {
                    country: Some(c.clone()),
                    holder_type: None,
                };
                if let Some(res) = reg.generate(&opts, &mut rng) {
                    history_results.push(res.code.clone());
                    rows.push(TaxIdRow {
                        code: res.code,
                        name: res.name,
                        country: format!("{} — {}", res.country_code, res.country_name),
                        holder_type: res.holder_type,
                        valid: res.valid,
                    });
                }
            }
        });
        results.set(rows);
        copied_idx.set(None);
        add_to_history("Tax ID", &c, n, history_results);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let text: String = rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,Name,Type,Country,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                row.code,
                row.name,
                row.holder_type.as_deref().unwrap_or(""),
                row.country,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("tax_ids.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("tax_ids.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS tax_ids (code TEXT, name TEXT, holder_type TEXT, country TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO tax_ids (code, name, holder_type, country, valid) VALUES ('{}', '{}', '{}', '{}', {});\n",
                row.code, row.name, row.holder_type.as_deref().unwrap_or(""), row.country, row.valid
            ));
        }
        download_file("tax_ids.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select: Vec<(String, String)> = countries
        .into_iter()
        .map(|(c, n, _)| (c, n))
        .collect();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"Name"</th>
                        <th>"Type"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let name = row.name.clone();
                            let holder_type = row.holder_type.clone().unwrap_or_default();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{name}</td>
                                    <td>{holder_type}</td>
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
fn VatTab() -> impl IntoView {
    let registry = vat::Registry::new();
    let countries: Vec<(String, String)> = registry
        .list_countries()
        .iter()
        .map(|(c, n)| (c.to_string(), n.to_string()))
        .collect();

    let country = RwSignal::new(
        countries
            .first()
            .map(|(c, _)| c.clone())
            .unwrap_or_default(),
    );
    let count = RwSignal::new(5u32);
    let results: RwSignal<Vec<VatRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let c = country.get();
        let n = count.get();
        let mut rows = Vec::new();
        let mut history_results = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = vat::GenOptions {
                    country: Some(c.clone()),
                };
                if let Some(res) = reg.generate(&opts, &mut rng) {
                    history_results.push(res.code.clone());
                    rows.push(VatRow {
                        code: res.code,
                        country_code: res.country_code,
                        country_name: res.country_name,
                        valid: res.valid,
                    });
                }
            }
        });
        results.set(rows);
        copied_idx.set(None);
        add_to_history("VAT", &c, n, history_results);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let text: String = rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,Country Code,Country Name,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                row.code,
                row.country_code,
                row.country_name,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("vat_numbers.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("vat_numbers.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS vat_numbers (code TEXT, country_code TEXT, country_name TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO vat_numbers (code, country_code, country_name, valid) VALUES ('{}', '{}', '{}', {});\n",
                row.code, row.country_code, row.country_name, row.valid
            ));
        }
        download_file("vat_numbers.sql", &sql, "text/plain;charset=utf-8;");
    };

    let countries_for_select = countries.clone();

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country"</label>
                <SearchableSelect
                    options=countries_for_select
                    selected=country
                    on_change=Callback::new(|_| ())
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
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
                        <th>"Country"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let country_display = format!("{} — {}", row.country_code, row.country_name);
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{country_display}</td>
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
fn LeiTab() -> impl IntoView {
    let registry = lei::Registry::new();

    let count = RwSignal::new(5u32);
    let country = RwSignal::new(String::new());
    let results: RwSignal<Vec<LeiRow>> = RwSignal::new(Vec::new());
    let copied_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    let registry = StoredValue::new(registry);

    let generate = move |_| {
        let mut rng = thread_rng();
        let n = count.get();
        let c = country.get();
        let mut rows = Vec::new();
        let mut history_results = Vec::new();
        registry.with_value(|reg| {
            for _ in 0..n {
                let opts = lei::GenOptions {
                    country: if c.is_empty() { None } else { Some(c.clone()) },
                };
                let res = reg.generate(&opts, &mut rng);
                history_results.push(res.code.clone());
                rows.push(LeiRow {
                    code: res.code,
                    lou: res.lou,
                    country_code: res.country_code,
                    valid: res.valid,
                });
            }
        });
        results.set(rows);
        copied_idx.set(None);
        add_to_history("LEI", if c.is_empty() { "Random" } else { &c }, n, history_results);
    };

    let copy_all = move |_| {
        let rows = results.get();
        let text: String = rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>().join("\n");
        copy_to_clipboard(&text);
    };

    let save_csv = move |_| {
        let rows = results.get();
        let mut csv = String::from("Code,LOU,Country,Valid\n");
        for row in rows.iter() {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                row.code,
                row.lou,
                row.country_code,
                if row.valid { "Yes" } else { "No" }
            ));
        }
        download_csv("lei_codes.csv", &csv);
    };

    let save_json = move |_| {
        let rows = results.get();
        let json = serde_json::to_string_pretty(&rows).unwrap_or_default();
        download_file("lei_codes.json", &json, "application/json;charset=utf-8;");
    };

    let save_sql = move |_| {
        let rows = results.get();
        let mut sql = String::from("CREATE TABLE IF NOT EXISTS lei_codes (code TEXT, lou TEXT, country_code TEXT, valid BOOLEAN);\n");
        for row in rows.iter() {
            sql.push_str(&format!(
                "INSERT INTO lei_codes (code, lou, country_code, valid) VALUES ('{}', '{}', '{}', {});\n",
                row.code, row.lou, row.country_code, row.valid
            ));
        }
        download_file("lei_codes.sql", &sql, "text/plain;charset=utf-8;");
    };

    view! {
        <div class="controls">
            <div class="field">
                <label>"Country (optional)"</label>
                <input type="text" placeholder="e.g. US (leave empty for random)"
                    prop:value=move || country.get()
                    on:input=move |ev| country.set(event_target_value(&ev))
                />
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

            <button class="btn btn-primary" on:click=generate>"Generate"</button>

            <Show when=move || !results.get().is_empty()>
                <button class="btn btn-secondary" on:click=copy_all>"Copy all"</button>
                <button class="btn btn-secondary" on:click=save_csv>"CSV"</button>
                <button class="btn btn-secondary" on:click=save_json>"JSON"</button>
                <button class="btn btn-secondary" on:click=save_sql>"SQL"</button>
            </Show>
        </div>

        <Show when=move || results.get().is_empty()>
            <div class="empty">"Click Generate to create LEI codes"</div>
        </Show>

        <Show when=move || !results.get().is_empty()>
            <div class="results-header">
                <span>{move || format!("{} results", results.get().len())}</span>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>"Code"</th>
                        <th>"LOU"</th>
                        <th>"Country"</th>
                        <th>"Valid"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let cidx = copied_idx.get();
                        results.get().iter().enumerate().map(|(i, row)| {
                            let code = row.code.clone();
                            let copy_text = code.clone();
                            let lou = row.lou.clone();
                            let country_code = row.country_code.clone();
                            let valid_class = if row.valid { "valid-yes" } else { "valid-no" };
                            let is_copied = cidx == Some(i);
                            view! {
                                <tr>
                                    <td>{code}</td>
                                    <td>{lou}</td>
                                    <td>{country_code}</td>
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
fn SearchableSelect(
    options: Vec<(String, String)>,
    selected: RwSignal<String>,
    on_change: Callback<()>,
) -> impl IntoView {
    let search_text = RwSignal::new(String::new());
    let is_open = RwSignal::new(false);
    let options = StoredValue::new(options);

    let filtered_options = Memo::new(move |_| {
        let query = search_text.get().to_lowercase();
        options.with_value(|opts| {
            if query.is_empty() {
                opts.clone()
            } else {
                opts.iter()
                    .filter(|(code, name)| {
                        code.to_lowercase().contains(&query) || name.to_lowercase().contains(&query)
                    })
                    .cloned()
                    .collect()
            }
        })
    });

    let display_name = Memo::new(move |_| {
        let current = selected.get();
        options.with_value(|opts| {
            opts.iter()
                .find(|(code, _)| code == &current)
                .map(|(_, name)| format!("{} \u{2014} {}", current, name))
                .unwrap_or_else(|| "Select country...".to_string())
        })
    });

    view! {
        <div class="searchable-select"
            on:focusout=move |_| {
                set_timeout(move || is_open.set(false), std::time::Duration::from_millis(200));
            }
        >
            <input type="text"
                class="search-input"
                placeholder=move || display_name.get()
                prop:value=move || search_text.get()
                on:input=move |ev| {
                    search_text.set(event_target_value(&ev));
                    is_open.set(true);
                }
                on:focus=move |_| is_open.set(true)
            />

            <Show when=move || is_open.get()>
                <div class="dropdown-results">
                    {move || {
                        let items = filtered_options.get();
                        if items.is_empty() {
                            view! { <div class="dropdown-item">"No results found"</div> }.into_any()
                        } else {
                            items.into_iter().map(|(code, name)| {
                                let code_c = code.clone();
                                let is_selected = selected.get() == code;
                                view! {
                                    <div
                                        class=format!("dropdown-item {}", if is_selected { "selected" } else { "" })
                                        on:click=move |_| {
                                            selected.set(code_c.clone());
                                            search_text.set(String::new());
                                            is_open.set(false);
                                            on_change.run(());
                                        }
                                    >
                                        {format!("{code} \u{2014} {name}")}
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ValidatorTab() -> impl IntoView {
    let input_value = RwSignal::new(String::new());
    let selected_type = RwSignal::new("iban".to_string());
    let country = RwSignal::new("DE".to_string());
    let result: RwSignal<Option<(bool, String)>> = RwSignal::new(None);

    let id_registry = personal_id::Registry::new();
    let bank_registry = bank_account::Registry::new();
    let card_registry = credit_card::Registry::new();
    let swift_registry = swift::Registry::new();
    let company_registry = company_id::Registry::new();
    let dl_registry = driver_license::Registry::new();
    let passport_registry = passport::Registry::new();
    let tax_registry = tax_id::Registry::new();
    let vat_registry = vat::Registry::new();
    let lei_registry = lei::Registry::new();

    let id_countries: Vec<(String, String)> = id_registry
        .list_countries()
        .iter()
        .map(|(c, n, _)| (c.to_string(), n.to_string()))
        .collect();

    let bank_countries: Vec<(String, String)> = bank_registry
        .list_countries()
        .iter()
        .map(|(c, n, _, _)| (c.to_string(), n.to_string()))
        .collect();

    let company_countries: Vec<(String, String)> = company_registry
        .list_countries()
        .iter()
        .map(|(c, n, _)| (c.to_string(), n.to_string()))
        .collect();

    let dl_countries: Vec<(String, String)> = dl_registry
        .list_countries()
        .iter()
        .map(|(c, n, _)| (c.to_string(), n.to_string()))
        .collect();

    let passport_countries: Vec<(String, String)> = passport_registry
        .list_countries()
        .iter()
        .map(|(c, n, _)| (c.to_string(), n.to_string()))
        .collect();

    let tax_countries: Vec<(String, String)> = tax_registry
        .list_countries()
        .iter()
        .map(|(c, n, _)| (c.to_string(), n.to_string()))
        .collect();

    let id_countries = StoredValue::new(id_countries);
    let bank_countries = StoredValue::new(bank_countries);
    let company_countries = StoredValue::new(company_countries);
    let dl_countries = StoredValue::new(dl_countries);
    let passport_countries = StoredValue::new(passport_countries);
    let tax_countries = StoredValue::new(tax_countries);

    let id_registry = StoredValue::new(id_registry);
    let bank_registry = StoredValue::new(bank_registry);
    let card_registry = StoredValue::new(card_registry);
    let swift_registry = StoredValue::new(swift_registry);
    let company_registry = StoredValue::new(company_registry);
    let dl_registry = StoredValue::new(dl_registry);
    let passport_registry = StoredValue::new(passport_registry);
    let tax_registry = StoredValue::new(tax_registry);
    let vat_registry = StoredValue::new(vat_registry);
    let lei_registry = StoredValue::new(lei_registry);

    let validate = move |_| {
        let val = input_value.get().trim().to_string();
        if val.is_empty() {
            result.set(None);
            return;
        }

        match selected_type.get().as_str() {
            "iban" => {
                let is_valid = iban::validate_iban(&val);
                result.set(Some((
                    is_valid,
                    if is_valid {
                        "Valid IBAN".to_string()
                    } else {
                        "Invalid IBAN checksum or format".to_string()
                    },
                )));
            }
            "id" => {
                id_registry.with_value(|reg| {
                    if let Some(parsed) = reg.parse(&country.get(), &val) {
                        if parsed.valid {
                            result.set(Some((
                                true,
                                format!(
                                    "Valid ID ({} / {})",
                                    parsed.gender.unwrap_or_default(),
                                    parsed.dob.unwrap_or_default()
                                ),
                            )));
                        } else {
                            result
                                .set(Some((false, "Invalid ID for selected country".to_string())));
                        }
                    } else {
                        result.set(Some((false, "Could not parse ID".to_string())));
                    }
                });
            }
            "bank" => {
                bank_registry.with_value(|reg| match reg.validate(&country.get(), &val) {
                    Some(true) => result.set(Some((
                        true,
                        "Valid Bank Account for selected country".to_string(),
                    ))),
                    Some(false) => result.set(Some((
                        false,
                        "Invalid Bank Account checksum or format".to_string(),
                    ))),
                    None => result.set(Some((
                        false,
                        "Unsupported country for Bank Account validation".to_string(),
                    ))),
                });
            }
            "card" => {
                card_registry.with_value(|reg| {
                    let is_valid = reg.validate(&val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid Credit Card (Luhn check passed)".to_string()
                        } else {
                            "Invalid Credit Card (Luhn check failed)".to_string()
                        },
                    )));
                });
            }
            "swift" => {
                swift_registry.with_value(|reg| {
                    let is_valid = reg.validate(&val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid SWIFT/BIC format".to_string()
                        } else {
                            "Invalid SWIFT/BIC format".to_string()
                        },
                    )));
                });
            }
            "company" => {
                company_registry.with_value(|reg| {
                    let is_valid = reg.validate(&country.get(), &val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid Company ID for selected country".to_string()
                        } else {
                            "Invalid Company ID checksum or format".to_string()
                        },
                    )));
                });
            }
            "driver_license" => {
                dl_registry.with_value(|reg| {
                    let is_valid = reg.validate(&country.get(), &val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid Driver's License for selected country".to_string()
                        } else {
                            "Invalid Driver's License format".to_string()
                        },
                    )));
                });
            }
            "passport" => {
                passport_registry.with_value(|reg| {
                    let is_valid = reg.validate(&country.get(), &val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid Passport for selected country".to_string()
                        } else {
                            "Invalid Passport format".to_string()
                        },
                    )));
                });
            }
            "tax_id" => {
                tax_registry.with_value(|reg| {
                    let is_valid = reg.validate(&country.get(), &val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid Tax ID for selected country".to_string()
                        } else {
                            "Invalid Tax ID format".to_string()
                        },
                    )));
                });
            }
            "vat" => {
                vat_registry.with_value(|reg| {
                    let is_valid = reg.validate(&val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid VAT number".to_string()
                        } else {
                            "Invalid VAT number format".to_string()
                        },
                    )));
                });
            }
            "lei" => {
                lei_registry.with_value(|reg| {
                    let is_valid = reg.validate(&val);
                    result.set(Some((
                        is_valid,
                        if is_valid {
                            "Valid LEI code".to_string()
                        } else {
                            "Invalid LEI code format".to_string()
                        },
                    )));
                });
            }
            _ => {}
        }
    };

    view! {
        <div class="validator-tab">
            <div class="controls">
                <div class="field">
                    <label>"Type"</label>
                    <select on:change=move |ev| {
                        let t = event_target_value(&ev);
                        selected_type.set(t.clone());
                        result.set(None);
                        match t.as_str() {
                            "id" => country.set("DE".to_string()),
                            "bank" => country.set("US".to_string()),
                            "company" | "driver_license" | "passport" | "tax_id" => country.set("EE".to_string()),
                            _ => {}
                        }
                    }>
                        <option value="iban">"IBAN"</option>
                        <option value="id">"Personal ID"</option>
                        <option value="bank">"Bank Account"</option>
                        <option value="card">"Credit Card"</option>
                        <option value="swift">"SWIFT/BIC"</option>
                        <option value="company">"Company ID"</option>
                        <option value="driver_license">"Driver's License"</option>
                        <option value="passport">"Passport"</option>
                        <option value="tax_id">"Tax ID"</option>
                        <option value="vat">"VAT"</option>
                        <option value="lei">"LEI"</option>
                    </select>
                </div>

                <Show when=move || {
                    let t = selected_type.get();
                    t == "id" || t == "bank" || t == "company" || t == "driver_license" || t == "passport" || t == "tax_id"
                }>
                    <div class="field">
                        <label>"Country"</label>
                        {move || {
                            let list = match selected_type.get().as_str() {
                                "id" => id_countries.get_value(),
                                "bank" => bank_countries.get_value(),
                                "company" => company_countries.get_value(),
                                "driver_license" => dl_countries.get_value(),
                                "passport" => passport_countries.get_value(),
                                "tax_id" => tax_countries.get_value(),
                                _ => Vec::new(),
                            };
                            view! {
                                <SearchableSelect
                                    options=list
                                    selected=country
                                    on_change=Callback::new(move |_| result.set(None))
                                />
                            }
                        }}
                    </div>
                </Show>

                <div class="field" style="flex: 1">
                    <label>"Value to validate"</label>
                    <input type="text"
                        placeholder="Enter code here..."
                        prop:value=move || input_value.get()
                        on:input=move |ev| input_value.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                validate(());
                            }
                        }
                    />
                </div>

                <button class="btn btn-primary" on:click=move |_| validate(())>"Validate"</button>
            </div>

            <div class="validator-result">
                {move || result.get().map(|(valid, msg)| {
                    let class = if valid { "result-valid" } else { "result-invalid" };
                    view! {
                        <div class=format!("result-box {}", class)>
                            <strong>{if valid { "VALID" } else { "INVALID" }}</strong>
                            <p>{msg}</p>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
