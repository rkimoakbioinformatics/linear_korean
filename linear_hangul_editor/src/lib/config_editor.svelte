<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { json } from "@codemirror/lang-json";
    import JSON5 from "json5";
    import CodeEditor from "$lib/code_editor.svelte";

    const JSON_LANGUAGE = json();
    const CONFIG_NUMERIC_FIELD_TYPES = {
        jung_w_ratio: "number",
        jong_w_ratio: "number",
        cho_h_ratio: "number",
        jung_h_ratio: "number",
        jong_h_ratio: "number",
        char_gap: "integer",
        cho_cho_gap: "integer",
        jung_jung_gap: "integer",
        jong_jong_gap: "integer",
        cho_jung_gap: "integer",
        jung_jong_gap: "integer",
        x_sw: "number",
        y_sw: "number",
        text_size: "integer",
        underdot_y: "integer",
        underdot_r_ratio: "number",
        upperdot_y: "integer",
        upperdot_r_ratio: "number",
        glyph_width: "integer",
        cap_height: "integer",
        x_height: "integer",
        baseline: "integer",
        min_gap: "integer",
        space_width_ratio: "number",
    };

    let config_data = $state("");
    let {
        config_name = $bindable(),
        theme_mode = "light",
        on_config_error = null,
    } = $props();
    let config_names = $state([]);
    let code_editor_ref = $state(null);

    onMount(async function () {
        await get_config_names(null);
        try {
            await load_config_data(null);
        } catch (e) {
            notify_config_error(e, "Config Load Error");
        }
    });

    async function get_config_names(event) {
        config_names = await invoke("get_config_names", {});
    }

    export async function load_config_data(event) {
        const loaded_config_data = await invoke("get_config_data", {
            configName: config_name,
        });
        config_data = loaded_config_data;
        validate_config_values(loaded_config_data, config_name || "default");
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
        validate_config_values(config_data, target_config_name);
        await invoke("save_config", {
            configData: config_data,
            configName: target_config_name,
        });
        config_name = target_config_name;
        await get_config_names(null);
    }

    function validate_config_values(source, config_label) {
        const raw = String(source ?? "").trim();
        if (raw == "") {
            return;
        }
        const parsed = JSON5.parse(raw);
        if (parsed == null || typeof parsed != "object" || Array.isArray(parsed)) {
            throw new Error(`Config '${config_label}' must be a JSON object.`);
        }
        const errors = [];
        for (const [field_name, expected_type] of Object.entries(
            CONFIG_NUMERIC_FIELD_TYPES,
        )) {
            if (!(field_name in parsed)) {
                continue;
            }
            const value = parsed[field_name];
            if (value == null) {
                errors.push(`'${field_name}' cannot be null.`);
                continue;
            }
            if (typeof value != "number" || !Number.isFinite(value)) {
                errors.push(`'${field_name}' must be a finite number.`);
                continue;
            }
            if (expected_type == "integer" && !Number.isInteger(value)) {
                errors.push(`'${field_name}' must be an integer.`);
            }
        }
        if (errors.length > 0) {
            throw new Error(
                `Invalid config values in '${config_label}':\n${errors.join("\n")}`,
            );
        }
    }

    function notify_config_error(error, title = "Error") {
        const message =
            error instanceof Error
                ? error.message
                : typeof error == "string"
                  ? error
                  : String(error);
        if (typeof on_config_error == "function") {
            on_config_error(title, message);
        }
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
