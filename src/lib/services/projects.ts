import { invoke } from '@tauri-apps/api/core';

export interface Project {
  id: string;
  name: string;
  path: string;
  context: string | null;
}

export async function getProjects(): Promise<Project[]> {
  return invoke<Project[]>('get_projects');
}

export async function addProject(path: string): Promise<Project[]> {
  return invoke<Project[]>('add_project', { path });
}

export async function removeProject(id: string): Promise<Project[]> {
  return invoke<Project[]>('remove_project', { id });
}
