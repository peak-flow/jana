<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

// A themed, editable model picker. Replaces the native <datalist> (which can't
// be styled) with a dark dropdown: type to filter, click to select, free text
// still allowed for models the list doesn't include.
const props = defineProps<{
  modelValue: string;
  options: string[];
  placeholder?: string;
  inputId?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
}>();

const open = ref(false);
const wrapper = ref<HTMLElement>();

const filtered = computed(() => {
  const q = props.modelValue.trim().toLowerCase();
  // Empty, or an exact match → show the full list so the user can switch freely.
  if (!q || props.options.some((o) => o.toLowerCase() === q)) return props.options;
  const matches = props.options.filter((o) => o.toLowerCase().includes(q));
  return matches.length ? matches : props.options;
});

function onInput(e: Event) {
  emit("update:modelValue", (e.target as HTMLInputElement).value);
  open.value = true;
}

function select(opt: string) {
  emit("update:modelValue", opt);
  open.value = false;
}

function onFocus() {
  if (props.options.length) open.value = true;
}

// Delay so a click on an option registers before the list closes.
function onBlur() {
  setTimeout(() => (open.value = false), 120);
}

// Close when clicking anywhere outside the component (covers the case where the
// list was opened programmatically after a fetch, with no input focus to blur).
function onDocMouseDown(e: MouseEvent) {
  if (open.value && wrapper.value && !wrapper.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => document.addEventListener("mousedown", onDocMouseDown));
onUnmounted(() => document.removeEventListener("mousedown", onDocMouseDown));

// Let the parent pop the list open after a "Fetch models" call.
defineExpose({ openList: () => (open.value = true) });
</script>

<template>
  <div ref="wrapper" class="combo">
    <input
      :id="inputId"
      :value="modelValue"
      type="text"
      autocomplete="off"
      :placeholder="placeholder"
      @input="onInput"
      @focus="onFocus"
      @blur="onBlur"
      @keydown.escape="open = false"
    />
    <ul v-if="open && filtered.length" class="combo-list">
      <li
        v-for="o in filtered"
        :key="o"
        class="combo-item"
        :class="{ active: o === modelValue }"
        @mousedown.prevent="select(o)"
      >
        {{ o }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.combo {
  position: relative;
  flex: 1;
}

.combo input {
  width: 100%;
  padding: 8px 10px;
  background: #181825;
  border: 1px solid #45475a;
  border-radius: 4px;
  color: #cdd6f4;
  font-size: 13px;
  box-sizing: border-box;
}

.combo input:focus {
  outline: none;
  border-color: #89b4fa;
}

.combo-list {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  z-index: 200;
  margin: 0;
  padding: 4px 0;
  list-style: none;
  max-height: 200px;
  overflow-y: auto;
  background: #1e1e2e;
  border: 1px solid #45475a;
  border-radius: 6px;
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.5);
}

.combo-item {
  padding: 7px 12px;
  font-size: 13px;
  color: #cdd6f4;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.combo-item:hover {
  background: #313244;
}

.combo-item.active {
  background: #45475a;
  color: #89b4fa;
}
</style>
