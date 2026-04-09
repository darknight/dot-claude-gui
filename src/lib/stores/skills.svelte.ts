import { connectionStore } from "./connection.svelte";
import { toastStore } from "./toast.svelte";
import type { SkillInfo } from "$lib/api/types";

class SkillsStore {
  skills = $state<SkillInfo[]>([]);
  selectedSkillId = $state<string | null>(null);
  skillContent = $state<string | null>(null);
  contentLoading = $state(false);
  loading = $state(false);
  error = $state<string>("");

  get selectedSkill(): SkillInfo | undefined {
    return this.skills.find((s) => s.id === this.selectedSkillId);
  }

  async loadSkills() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    try {
      this.skills = await client.listSkills();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load skills";
    } finally {
      this.loading = false;
    }
  }

  selectSkill(id: string | null) {
    this.selectedSkillId = id;
    this.skillContent = null;
    if (id) {
      void this.loadSkillContent(id);
    }
  }

  async loadSkillContent(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.contentLoading = true;
    try {
      const res = await client.getSkillContent(id);
      this.skillContent = res.content;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Failed to load skill content";
      toastStore.error(msg);
      this.skillContent = null;
    } finally {
      this.contentLoading = false;
    }
  }

  reset(): void {
    this.skills = [];
    this.selectedSkillId = null;
    this.skillContent = null;
    this.contentLoading = false;
    this.loading = false;
    this.error = "";
  }
}

export const skillsStore = new SkillsStore();
