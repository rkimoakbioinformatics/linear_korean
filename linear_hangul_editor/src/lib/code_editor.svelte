<script>
    // @ts-nocheck
    import { onMount } from "svelte";
    import { Compartment, EditorState } from "@codemirror/state";
    import { EditorView } from "@codemirror/view";
    import { basicSetup } from "codemirror";
    import { oneDark } from "@codemirror/theme-one-dark";

    let {
        value = $bindable(""),
        language_extension = [],
        theme_mode = "light",
        min_height = "14rem",
        read_only = false,
        aria_label = "Code editor",
    } = $props();

    let host = $state(null);
    let editor = null;

    const language_compartment = new Compartment();
    const theme_compartment = new Compartment();
    const editable_compartment = new Compartment();

    const light_editor_theme = EditorView.theme({
        "&": {
            height: "100%",
            fontSize: "0.9rem",
        },
        ".cm-scroller": {
            fontFamily:
                "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace",
            lineHeight: "1.45",
        },
        ".cm-content": {
            padding: "0.75rem",
        },
    });

    const dark_editor_theme = EditorView.theme(
        {
            "&": {
                height: "100%",
            },
            ".cm-content": {
                padding: "0.75rem",
            },
        },
        { dark: true },
    );

    function get_theme_extensions() {
        if (theme_mode == "dark") {
            return [oneDark, dark_editor_theme];
        }
        return [light_editor_theme];
    }

    function create_editor() {
        if (!host) {
            return;
        }
        editor = new EditorView({
            state: EditorState.create({
                doc: value,
                extensions: [
                    basicSetup,
                    EditorView.lineWrapping,
                    language_compartment.of(language_extension),
                    theme_compartment.of(get_theme_extensions()),
                    editable_compartment.of(EditorView.editable.of(!read_only)),
                    EditorView.updateListener.of((update) => {
                        if (!update.docChanged) {
                            return;
                        }
                        const next_value = update.state.doc.toString();
                        if (next_value != value) {
                            value = next_value;
                        }
                    }),
                ],
            }),
            parent: host,
        });
    }

    export function focusEditor() {
        editor?.focus();
    }

    onMount(() => {
        create_editor();
        return () => {
            editor?.destroy();
            editor = null;
        };
    });

    $effect(() => {
        if (!editor) {
            return;
        }
        const current_text = editor.state.doc.toString();
        if (current_text == value) {
            return;
        }
        editor.dispatch({
            changes: {
                from: 0,
                to: current_text.length,
                insert: value,
            },
        });
    });

    $effect(() => {
        if (!editor) {
            return;
        }
        editor.dispatch({
            effects: language_compartment.reconfigure(language_extension),
        });
    });

    $effect(() => {
        if (!editor) {
            return;
        }
        editor.dispatch({
            effects: theme_compartment.reconfigure(get_theme_extensions()),
        });
    });

    $effect(() => {
        if (!editor) {
            return;
        }
        editor.dispatch({
            effects: editable_compartment.reconfigure(
                EditorView.editable.of(!read_only),
            ),
        });
    });
</script>

<div
    bind:this={host}
    class="h-full min-h-0 overflow-hidden rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))]"
    style={`min-height: ${min_height};`}
    role="textbox"
    aria-label={aria_label}
></div>
