import { connectionStore } from "./connection.svelte";
import type { SkillInfo } from "$lib/api/types";

class SkillsStore {
  skills = $state<SkillInfo[]>([]);
  selectedSkillId = $state<string | null>(null);
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
  }

  reset(): void {
    this.skills = [];
    this.selectedSkillId = null;
    this.loading = false;
    this.error = "";
  }
}

export const skillsStore = new SkillsStore();
