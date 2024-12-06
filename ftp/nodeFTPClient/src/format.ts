import type { IFile } from "./type";

export function formatResponse(json: any): IFile[] {
  try {
    const x=  json.map((file: any) => {
      const formattedFile: IFile = {
        name: file.name,
        type: file.type === "d" ? "directory" : "file",
      };

      if (file.size) {
        formattedFile.size = formatSize(file.size);
      }

      if (file.date) {
        formattedFile.date = new Date(file.date).toISOString().split("T")[0];
      }

      return formattedFile;
    });
    return x;

  } catch (error) {
    console.error("Error formatting file:", error);
    return [];
  }
}

function formatSize(size: number): string {
  try{

    const i = Math.floor(Math.log(size) / Math.log(1024));
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    return (size / Math.pow(1024, i)).toFixed(1) + " " + sizes[i];
  } catch (error) {
    throw new Error("Error formatting size: " + error);
  }
}
