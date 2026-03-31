<script lang="ts">
  import { connectionsStore } from "$lib/stores/connections.svelte.js";
  import { DaemonClient } from "$lib/api/client.js";
  import type { ConnectionEntry } from "$lib/api/types.js";

  let editing = $state<ConnectionEntry | null>(null);
  let isNew = $state(false);

  let formName = $state("");
  let formUrl = $state("");
  let formToken = $state("");
  let showToken = $state(false);

  let testStatus = $state<"idle" | "testing" | "ok" | "error">("idle");
  let testMessage = $state("");

  function startAdd() {
    isNew = true;
    formName = "";
    formUrl = "http://";
    formToken = "";
    testStatus = "idle";
    editing = {} as ConnectionEntry;
  }

  function startEdit(entry: ConnectionEntry) {
    isNew = false;
    formName = entry.name;
    formUrl = entry.url;
    formToken = entry.token;
    testStatus = "idle";
    editing = entry;
  }

  function cancelEdit() {
    editing = null;
    isNew = false;
  }

  async function saveEdit() {
    if (isNew) {
      await connectionsStore.addConnection({
        name: formName,
        type: "remote",
        url: formUrl.replace(/\/$/, ""),
        token: formToken,
        managed: false,
      });
    } else if (editing) {
      await connectionsStore.updateConnection(editing.id, {
        name: formName,
        url: formUrl.replace(/\/$/, ""),
        token: formToken,
      });
    }
    editing = null;
    isNew = false;
  }

  async function deleteConnection(id: string) {
    await connectionsStore.deleteConnection(id);
  }

  async function testConnection() {
    testStatus = "testing";
    testMessage = "";
    try {
      const client = new DaemonClient(formUrl.replace(/\/$/, ""), formToken);
      const health = await client.health();
      testStatus = "ok";
      testMessage = `连接成功 (v${health.version})`;
    } catch (err) {
      testStatus = "error";
      testMessage = err instanceof Error ? err.message : String(err);
    }
  }
</script>

<div class="space-y-4">
  <h2 class="text-lg font-medium text-gray-100">连接管理</h2>

  <div class="space-y-2">
    {#each connectionsStore.connections as entry}
      <div class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
        <div>
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-gray-200">{entry.name}</span>
            {#if entry.managed}
              <span class="text-xs px-1.5 py-0.5 bg-gray-700 text-gray-400 rounded">自动管理</span>
            {/if}
          </div>
          <div class="text-xs text-gray-500 mt-0.5">{entry.url}</div>
        </div>
        <div class="flex gap-2">
          {#if !entry.managed}
            <button
              class="px-2 py-1 text-xs text-gray-400 hover:text-gray-200"
              onclick={() => startEdit(entry)}
            >
              编辑
            </button>
            <button
              class="px-2 py-1 text-xs text-red-400 hover:text-red-300"
              onclick={() => deleteConnection(entry.id)}
            >
              删除
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if !editing}
    <button
      class="px-3 py-2 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-500"
      onclick={startAdd}
    >
      + 添加连接
    </button>
  {/if}

  {#if editing}
    <div class="p-4 bg-gray-800 rounded-lg border border-gray-700 space-y-3">
      <h3 class="text-sm font-medium text-gray-200">
        {isNew ? "新建连接" : "编辑连接"}
      </h3>

      <div>
        <label class="block text-xs text-gray-400 mb-1">名称</label>
        <input
          type="text"
          bind:value={formName}
          placeholder="Docker Dev"
          class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
        />
      </div>

      <div>
        <label class="block text-xs text-gray-400 mb-1">URL</label>
        <input
          type="text"
          bind:value={formUrl}
          placeholder="http://192.168.1.100:7890"
          class="w-full bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
        />
      </div>

      <div>
        <label class="block text-xs text-gray-400 mb-1">Token</label>
        <div class="flex gap-2">
          <input
            type={showToken ? "text" : "password"}
            bind:value={formToken}
            class="flex-1 bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200"
          />
          <button
            class="px-2 text-sm text-gray-400 hover:text-gray-200"
            onclick={() => (showToken = !showToken)}
          >
            {showToken ? "隐藏" : "显示"}
          </button>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <button
          class="px-3 py-1.5 text-sm bg-gray-700 text-gray-200 rounded hover:bg-gray-600"
          onclick={testConnection}
          disabled={testStatus === "testing"}
        >
          {testStatus === "testing" ? "测试中..." : "测试连接"}
        </button>
        {#if testStatus === "ok"}
          <span class="text-sm text-green-400">{testMessage}</span>
        {:else if testStatus === "error"}
          <span class="text-sm text-red-400">{testMessage}</span>
        {/if}
      </div>

      <div class="flex justify-end gap-2 pt-2 border-t border-gray-700">
        <button
          class="px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200"
          onclick={cancelEdit}
        >
          取消
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-500"
          onclick={saveEdit}
          disabled={!formName.trim() || !formUrl.trim()}
        >
          保存
        </button>
      </div>
    </div>
  {/if}
</div>
