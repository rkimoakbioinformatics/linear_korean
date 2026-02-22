<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { json } from "@codemirror/lang-json";
    import JSON5 from "json5";
    import CodeEditor from "$lib/code_editor.svelte";

    const JSON_LANGUAGE = json();

    let config_data = $state("");
    let { config_name = $bindable(), theme_mode = "light" } = $props();
    let config_names = $state([]);
    let code_editor_ref = $state(null);

    onMount(async function () {
        await get_config_names(null);
        await load_config_data(null);
    });

    async function get_config_names(event) {
        config_names = await invoke("get_config_names", {});
    }

    export async function load_config_data(event) {
        config_data = await invoke("get_config_data", {
            configName: config_name,
        });
    }

    function format_config_json(source) {
        const parsed = JSON5.parse(source);
        let formatted = JSON.stringify(parsed, null, 2);
        if (!formatted.endsWith("\n")) {
            formatted += "\n";
        }
        return formatted;
    }

    export function focusEditor() {
        code_editor_ref?.focusEditor();
    }

    export function setConfigData(next_config_data) {
        config_data = String(next_config_data ?? "");
    }

    export function getConfigData() {
        return config_data;
    }

    export async function save(event, next_config_name = null) {
        const target_config_name =
            next_config_name == null ? config_name : next_config_name.trim();
        if (target_config_name == "") {
            return;
        }
        config_data = format_config_json(config_data);
        await invoke("save_config", {
            configData: config_data,
            configName: target_config_name,
        });
        config_name = target_config_name;
        await get_config_names(null);
    }
</script>

<div class="ui-card flex min-h-0 flex-1 flex-col p-3">
    <div class="mb-2 flex items-center justify-between">
        <div class="ui-label">Config Editor</div>
        <div class="text-xs text-[hsl(var(--muted-foreground))]">JSON</div>
    </div>
    <div class="min-h-0 flex-1">
        <CodeEditor
            bind:this={code_editor_ref}
            bind:value={config_data}
            language_extension={JSON_LANGUAGE}
            theme_mode={theme_mode}
            min_height="13rem"
            aria_label="Config editor"
        />
    </div>
</div>
