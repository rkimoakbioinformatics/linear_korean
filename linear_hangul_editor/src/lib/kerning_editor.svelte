<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { StreamLanguage } from "@codemirror/language";
    import { spreadsheet } from "@codemirror/legacy-modes/mode/spreadsheet";
    import CodeEditor from "$lib/code_editor.svelte";

    const CSV_LANGUAGE = StreamLanguage.define(spreadsheet);

    let kerning_data = $state("");
    let kerning_names = $state([]);
    let { kerning_name = $bindable(), theme_mode = "light" } = $props();
    let code_editor_ref = $state(null);

    onMount(async function () {
        await getKerningNames(null);
        await load_kerning_data(null);
    });

    async function getKerningNames(event) {
        kerning_names = await invoke("get_kerning_names", {});
    }

    export async function load_kerning_data(event) {
        kerning_data = await invoke("get_kerning_data", {
            kerningName: kerning_name,
        });
    }

    function sort_kerning_text(source) {
        const has_trailing_newline = /\r?\n$/.test(source);
        const rows = source
            .split(/\r?\n/)
            .map((line) => line.trimEnd())
            .filter((line) => line != "");
        rows.sort((a, b) => a.localeCompare(b));
        let sorted = rows.join("\n");
        if (has_trailing_newline && sorted != "") {
            sorted += "\n";
        }
        return sorted;
    }

    export function focusEditor() {
        code_editor_ref?.focusEditor();
    }

    export function setKerningData(next_kerning_data) {
        kerning_data = String(next_kerning_data ?? "");
    }

    export function getKerningData() {
        return kerning_data;
    }

    function sort_kerning_rows(event) {
        kerning_data = sort_kerning_text(kerning_data);
    }

    export async function save(event, next_kerning_name = null) {
        const target_kerning_name =
            next_kerning_name == null ? kerning_name : next_kerning_name.trim();
        if (target_kerning_name == "") {
            return;
        }
        kerning_data = sort_kerning_text(kerning_data);
        await invoke("save_kerning_data", {
            kerningData: kerning_data,
            kerningName: target_kerning_name,
        });
        kerning_name = target_kerning_name;
        await getKerningNames(null);
    }
</script>

<div class="ui-card flex min-h-0 flex-1 flex-col p-3">
    <div class="mb-2 flex items-center justify-between">
        <div class="ui-label">Kerning Editor</div>
        <!--<button class="ui-button ui-button-ghost text-xs" onclick={sort_kerning_rows}
            >Sort</button
        >-->
    </div>
    <div class="min-h-0 flex-1">
        <CodeEditor
            bind:this={code_editor_ref}
            bind:value={kerning_data}
            language_extension={CSV_LANGUAGE}
            theme_mode={theme_mode}
            min_height="13rem"
            aria_label="Kerning editor"
        />
    </div>
</div>
