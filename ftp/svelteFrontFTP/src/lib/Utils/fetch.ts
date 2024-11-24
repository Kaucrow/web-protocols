import type { IFile } from "@/types";
import { IFileSchema } from "./schemas";

const headers = {
  "Content-Type": "application/json",
};

const host = "localhost";
const port = "3040";
const baseUrl = `http://${host}:${port}`;

export async function connect(host: string, user: string, password: string) {
  try {
    const body = JSON.stringify({ host, user, password });
    const method = "POST";
    console.log("connect...");
    const response = await fetch(`${baseUrl}/api/connect`, {
      method,
      headers,
      body,
    });
    const data = await response.json();
    console.log("connect", data);
    return data;
  } catch (error) {
    return { success: false, error: error };
  }
}

export async function disconnect() {
  const method = "GET";
  console.log("disconnect...");
  const response = await fetch(`${baseUrl}/api/disconnect`, {
    method,
    headers,
  });
  const data = await response.json();
  console.log("disconnect", data);
  return data;
}

export async function getClientFiles() {
  try {
    const method = "GET";
    console.log("getClientFiles...");
    const response = await fetch(`${baseUrl}/api/get-client-files`, {
      headers,
      method,
    });
    const data = await response.json();
    console.log("getClientFiles", data);
    const files: IFile[] = data.files.map((file: any) =>
      IFileSchema.parse(file)
    );
    return files;
  } catch (error) {
    console.error(error);
    return [];
  }
}

export async function getServerFiles() {
  try {
    const method = "GET";
    console.log("getServerFiles...");
    const response = await fetch(`${baseUrl}/api/get-server-files`, {
      headers,
      method,
    });
    const data = await response.json();
    console.log("getServerFiles", data);
    if (!data.success) return [];
    const files: IFile[] = data.files.map((file: any) =>
      IFileSchema.parse(file)
    );
    return files;
  } catch (error) {
    console.error(error);
    return [];
  }
}

export async function downloadServerFile(path: string) {
  const body = JSON.stringify({ path });
  const method = "POST";
  const response = await fetch(`${baseUrl}/api/download-server-file`, {
    method,
    headers,
    body,
  });
  const data = await response.json();
  return data;
}

export async function sendFile(path: string) {
  const body = JSON.stringify({ path });
  const method = "POST";
  const response = await fetch(`${baseUrl}/api/send-file`, {
    method,
    headers,
    body,
  });
  const data = await response.json();
  return data;
}

export async function deleteFile(path: string) {
  const body = JSON.stringify({ path });
  const method = "DELETE";
  const response = await fetch(`${baseUrl}/api/delete-file`, {
    method,
    headers,
    body,
  });
  const data = await response.json();
  return data;
}

export async function createDirectory(path: string) {
  const body = JSON.stringify({ path });
  const method = "POST";
  const response = await fetch(`${baseUrl}/api/create-directory`, {
    method,
    headers,
    body,
  });
  const data = await response.json();
  return data;
}


