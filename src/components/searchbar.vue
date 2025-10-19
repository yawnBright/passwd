<template>
  <div class="search-bar">
    <div class="search-container">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"></circle>
        <path d="m21 21-4.35-4.35"></path>
      </svg>
      <input
        type="text"
        class="search-input"
        :placeholder="placeholder"
        :value="modelValue"
        @input="handleInput"
        @keyup.enter="handleSearch"
      />
      <button v-if="modelValue" class="clear-btn" @click="clearSearch">
        <svg class="clear-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'

interface Props {
  modelValue: string
  placeholder?: string
}

interface Emits {
  (e: 'update:modelValue', value: string): void
  (e: 'search', value: string): void
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '搜索...'
})

const emit = defineEmits<Emits>()

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

const handleSearch = () => {
  emit('search', props.modelValue)
}

const clearSearch = () => {
  emit('update:modelValue', '')
  emit('search', '')
}
</script>

<style scoped>
.search-bar {
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
}

.search-container {
  position: relative;
  display: flex;
  align-items: center;
  background: white;
  border-radius: 40px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border: 1px solid #e0e0e0;
  transition: all 0.3s ease;
}

.search-container:hover {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  border-color: #d0d0d0;
}

.search-container:focus-within {
  box-shadow: 0 0 0 3px rgba(66, 153, 225, 0.1);
  border-color: #4299e1;
}

.search-icon {
  width: 20px;
  height: 20px;
  margin-left: 16px;
  color: #9ca3af;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  padding: 16px;
  font-size: 16px;
  background: transparent;
  color: #374151;
}

.search-input::placeholder {
  color: #9ca3af;
}

.clear-btn {
  background: none;
  border: none;
  padding: 8px;
  margin-right: 8px;
  cursor: pointer;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s ease;
}

.clear-btn:hover {
  background-color: #f3f4f6;
}

.clear-icon {
  width: 16px;
  height: 16px;
  color: #9ca3af;
}

@media (prefers-color-scheme: dark) {
  .search-container {
    background: #1f2937;
    border-color: #374151;
  }
  
  .search-container:hover {
    border-color: #4b5563;
  }
  
  .search-input {
    color: #f9fafb;
  }
  
  .search-input::placeholder {
    color: #6b7280;
  }
  
  .clear-btn:hover {
    background-color: #374151;
  }
  
  .clear-icon {
    color: #6b7280;
  }
}

@media (max-width: 640px) {
  .search-bar {
    max-width: 100%;
    padding: 0 16px;
  }
  
  .search-input {
    font-size: 14px;
    padding: 12px;
  }
  
  .search-icon {
    width: 18px;
    height: 18px;
    margin-left: 12px;
  }
}
</style>