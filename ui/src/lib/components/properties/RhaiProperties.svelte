<script>
  import CodeMirror from "svelte-codemirror-editor";
  import { rust } from "@codemirror/lang-rust";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { EditorView } from "@codemirror/view";
  import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
  import { tags } from "@lezer/highlight";
  import { theme } from "../../stores/theme";
  import { Button } from '../ui';
  
  export let data = {};
  export let originalCode = '';
  export let onUpdate;
  
  // Light theme highlighting
  const lightHighlightStyle = HighlightStyle.define([
    { tag: tags.keyword, color: "#d73a49" },
    { tag: tags.variableName, color: "#24292e" },
    { tag: tags.propertyName, color: "#005cc5" },
    { tag: tags.string, color: "#032f62" },
    { tag: tags.number, color: "#005cc5" },
    { tag: tags.comment, color: "#6a737d", fontStyle: "italic" },
    { tag: tags.function(tags.variableName), color: "#6f42c1" },
    { tag: tags.operator, color: "#d73a49" },
    { tag: tags.punctuation, color: "#24292e" },
  ]);
  
  const lightTheme = [
    EditorView.theme({
      "&": { backgroundColor: "#ffffff", color: "#24292e" },
      ".cm-content": { caretColor: "#24292e" },
      ".cm-gutters": { backgroundColor: "#f6f8fa", color: "#6a737d" },
    }, { dark: false }),
    syntaxHighlighting(lightHighlightStyle)
  ];

  $: editorTheme = $theme === "dark" ? oneDark : lightTheme;
  $: isDirty = (data.code || "").trim() !== (originalCode || "").trim();
  
  function handleSave() {
    onUpdate(data);
  }
  
  function handleDiscard() {
    data.code = originalCode || "";
  }
</script>

<div class="flex flex-col h-[300px] border border-[var(--color-border)] rounded overflow-hidden">
  <label class="label px-2 py-1 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]">
    Rhai Script
  </label>
  <div class="flex-1 overflow-auto">
    <CodeMirror
      bind:value={data.code}
      lang={rust()}
      theme={editorTheme}
    />
  </div>
  <div class="px-2 py-1 bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)] flex justify-between items-center text-xs">
    <span class="text-[var(--color-text-muted)] italic">msg: MessagePayload is input</span>
    <div class="flex gap-2">
      <Button variant="secondary" size="sm" disabled={!isDirty} on:click={handleDiscard}>
        Discard
      </Button>
      <Button variant="primary" size="sm" disabled={!isDirty} on:click={handleSave}>
        Save Script
      </Button>
    </div>
  </div>
</div>
