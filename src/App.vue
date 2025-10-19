<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import SearchBar from "./components/searchbar.vue";

const greetMsg = ref("");
const name = ref("");
const searchQuery = ref("");

interface Data {
  val: string,
  map: Map<number, string>
}

let data: Data = {
  val: "hello",
  map: new Map([[1, "a"], [2, "b"]])
}

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
  data = await invoke("fetch_data");
}

const handleSearch = (query: string) => {
  console.log('搜索内容:', query);
  // 这里可以添加实际的搜索逻辑
  // 例如调用 Tauri 命令进行搜索
};
</script>

<template>
  <main class="container">
    <button @click="greet">Greet</button>
    <p>{{ data.val }}</p>
    <!-- 顶部搜索栏 -->
    <div style="padding: 20px 0;">
      <SearchBar 
        v-model="searchQuery" 
        placeholder="搜索密码..."
        @search="handleSearch"
      />
    </div>
    
    <!-- 内容列表 -->
    <div class="content-section">
      <!-- 这里可以显示搜索结果或其他内容 -->
    </div>
  </main>
</template>

<style scoped>
/* .search-section {
  padding: 20px 0;
  border-radius: 0 0 50px 50px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
} */

.content-section {
  padding: 20px;
  min-height: 400px;
}
</style>

<style root>
.container {
  background-color: white;
}
</style>
