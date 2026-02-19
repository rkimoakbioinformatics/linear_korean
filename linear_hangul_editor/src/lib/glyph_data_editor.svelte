<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { StreamLanguage } from "@codemirror/language";
    import { lua } from "@codemirror/legacy-modes/mode/lua";
    import CodeEditor from "$lib/code_editor.svelte";
    import initStylua, {
        format as styluaFormat,
        IndentType,
        LineEndings,
        OutputVerification,
        QuoteStyle,
    } from "stylua-wasm";
    import styluaWasmUrl from "stylua-wasm/stylua_wasm_bg.wasm?url";

    const LUA_LANGUAGE = StreamLanguage.define(lua);

    const GLYPH_OPTIONS = [
        { value: "gieug", label: "ㄱ" },
        { value: "ssang_gieug", label: "ㄲ" },
        { value: "nieun", label: "ㄴ" },
        { value: "dieud", label: "ㄷ" },
        { value: "ssang_dieud", label: "ㄸ" },
        { value: "lieul", label: "ㄹ" },
        { value: "mieum", label: "ㅁ" },
        { value: "bieub", label: "ㅂ" },
        { value: "ssang_bieub", label: "ㅃ" },
        { value: "sieus", label: "ㅅ" },
        { value: "ssang_sieus", label: "ㅆ" },
        { value: "ieung", label: "ㅇ" },
        { value: "yesieung", label: "iㅇ" },
        { value: "jieuj", label: "ㅈ" },
        { value: "ssang_jieuj", label: "ㅉ" },
        { value: "chieuch", label: "ㅊ" },
        { value: "kieuk", label: "ㅋ" },
        { value: "tieut", label: "ㅌ" },
        { value: "pieup", label: "ㅍ" },
        { value: "hieuh", label: "ㅎ" },
        { value: "a", label: "ㅏ" },
        { value: "ya", label: "ㅑ" },
        { value: "eo", label: "ㅓ" },
        { value: "yeo", label: "ㅕ" },
        { value: "o", label: "ㅗ" },
        { value: "yo", label: "ㅛ" },
        { value: "u", label: "ㅜ" },
        { value: "yu", label: "ㅠ" },
        { value: "eu", label: "ㅡ" },
        { value: "i", label: "ㅣ" },
        { value: "ae", label: "ㅐ" },
        { value: "eoe", label: "ㅔ" },
        { value: "yeoe", label: "ㅖ" },
        { value: "yae", label: "ㅒ" },
    ];

    let glyph_name = $state("gieug");
    let glyph_data = $state("");
    let prev_glyph_name = $state("");
    let loaded_glyph_set = $state("");
    let stylua_init_promise = null;

    let {
        glyph_set,
        editor_label = "Glyph Editor",
        theme_mode = "light",
    } = $props();

    let code_editor_ref = $state(null);
    let glyph_selector_ref = $state(null);
    const selector_id = $derived(
        `glyph-selector-${editor_label.toLowerCase().replaceAll(/[^a-z0-9]+/g, "-")}`,
    );

    async function ensure_stylua() {
        if (!stylua_init_promise) {
            stylua_init_promise = initStylua(styluaWasmUrl);
        }
        await stylua_init_promise;
    }

    async function format_glyph_lua(source) {
        await ensure_stylua();
        return styluaFormat(
            source,
            {
                line_endings: LineEndings.Unix,
                indent_type: IndentType.Spaces,
                indent_width: 2,
                quote_style: QuoteStyle.AutoPreferDouble,
                column_width: 100,
            },
            OutputVerification.Full,
        );
    }

    async function loadGlyphData(event) {
        if (glyph_set == "" || glyph_name == "") {
            return;
        }
        if (
            prev_glyph_name != "" &&
            glyph_data != "" &&
            glyph_data != "No data" &&
            loaded_glyph_set != ""
        ) {
            await invoke("save_glyph_data", {
                glyphSet: loaded_glyph_set,
                glyphName: prev_glyph_name,
                glyphData: glyph_data,
            });
        }
        glyph_data = await invoke("get_glyph_data", {
            glyphSet: glyph_set,
            glyphName: glyph_name,
        });
        prev_glyph_name = glyph_name;
        loaded_glyph_set = glyph_set;
    }

    export async function loadGlyphDataWithoutSave(event) {
        if (glyph_set == "" || glyph_name == "") {
            return;
        }
        glyph_data = await invoke("get_glyph_data", {
            glyphSet: glyph_set,
            glyphName: glyph_name,
        });
        prev_glyph_name = glyph_name;
        loaded_glyph_set = glyph_set;
    }

    export function focusEditor() {
        code_editor_ref?.focusEditor();
    }

    export function focusGlyphSelector() {
        if (!glyph_selector_ref) {
            return;
        }
        glyph_selector_ref.focus();
        glyph_selector_ref.click();
    }

    export async function save(event) {
        const target_glyph_set =
            loaded_glyph_set == "" ? glyph_set : loaded_glyph_set;
        if (
            target_glyph_set == "" ||
            glyph_name == "" ||
            glyph_data == "" ||
            glyph_data == "No data"
        ) {
            return;
        }
        glyph_data = await format_glyph_lua(glyph_data);
        await invoke("save_glyph_data", {
            glyphSet: target_glyph_set,
            glyphName: glyph_name,
            glyphData: glyph_data,
        });
    }
</script>

<div class="ui-card flex min-h-0 flex-col p-3">
    <div class="mb-2 flex items-center justify-between gap-2">
        <label class="ui-label" for={selector_id}>{editor_label}</label>
        <select
            bind:this={glyph_selector_ref}
            id={selector_id}
            class="ui-select h-8 text-sm"
            bind:value={glyph_name}
            onchange={loadGlyphData}
        >
            {#each GLYPH_OPTIONS as option}
                <option value={option.value}>{option.label}</option>
            {/each}
        </select>
    </div>
    <div class="min-h-0 flex-1">
        <CodeEditor
            bind:this={code_editor_ref}
            bind:value={glyph_data}
            language_extension={LUA_LANGUAGE}
            theme_mode={theme_mode}
            min_height="13rem"
            aria_label={`${editor_label} lua editor`}
        />
    </div>
</div>
