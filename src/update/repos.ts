import { updateJavaRender } from "./update.ts";

export const repos = {
    java: {
        render: "https://api.github.com/repos/TeamMV/Rendering"
    },
    ts: {

    }
};


export function getRepo(language: string, framework: string) {
    if (framework == "none" || framework == "") {
        return "";
    }

    switch (language) {
        case "java":
            switch (framework) {
                case "render":
                    return repos.java.render;
            }
            break;
        case "ts":
            break;
    }

    return "";
}

export async function updateVersion(language: string, framework: string) {
    if (framework == "none") {
        return;
    }

    switch (language) {
        case "java":
            switch (framework) {
                case "render":
                    await updateJavaRender();
            }
            break;
        case "ts":
            break;
    }
}

export interface Author {
    login: string;
    id: 1;
    node_id: string;
    avatar_url: string;
    gravatar_id: string;
    url: string;
    html_url: string;
    followers_url: string;
    following_url: string;
    gists_url: string;
    starred_url: string;
    subscriptions_url: string;
    organizations_url: string;
    repos_url: string;
    events_url: string;
    received_events_url: string;
    type: string;
    site_admin: boolean;
}

export interface Asset {
    url: string;
    browser_download_url: string;
    id: number;
    node_id: string;
    name: string;
    label: string;
    state: string;
    content_type: string;
    size: number;
    download_count: number;
    created_at: string;
    updated_at: string;
    uploader: Author;
}

export interface Release {
    url: string;
  html_url: string;
  assets_url: string;
  upload_url: string;
  tarball_url: string;
  zipball_url: string;
  discussion_url: string;
  id: 1;
  node_id: string;
  tag_name: string;
  target_commitish: string;
  name: string;
  body: string;
  draft: boolean;
  prerelease: boolean;
  created_at: string;
  published_at: string;
  author: Author;
  assets: Asset[];
}

export interface Versions {
    mvc: string;
    java: {
        rendering: string;
    }
}

export async function getVersion(language: string, framework: string) {
    if (framework == "none") {
        return "-1";
    }
    const res = await fetch(`https://files.mvteam.dev/version`, {
        method: "GET"
    });
    const versions: Versions =  JSON.parse(await (await res.blob()).text());
    switch (language) {
        case "java":
            switch (framework) {
                case "rendering":
                    return versions.java.rendering;
            }
            break;
        case "ts": 
            break;
    }
    return "-1";
}

export async function getToolVersion() {
    const res = await fetch(`https://files.mvteam.dev/version`, {
        method: "GET"
    });
    const versions: Versions =  JSON.parse(await (await res.blob()).text());
    return versions.mvc;
}

export async function getRelease(repo: string) {
    const res = await fetch(`${repo}/releases/latest`, {
        method: "GET"
    });
    return await res.json() as Release;
}