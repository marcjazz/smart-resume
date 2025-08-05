import latex from "node-latex";
import { Readable } from "stream";

/**
 * Generate a PDF Buffer from a LaTeX template string.
 * @param content LaTeX source string.
 * @returns Promise resolving to PDF Buffer.
 */
export async function generatePdf(content: string): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const input = Readable.from([content]);
    const options = { args: ["-interaction=nonstopmode"] };
    const pdf = latex(input, options);
    const chunks: Buffer[] = [];
    pdf.on("data", (chunk: Buffer) => chunks.push(chunk));
    pdf.on("error", (err) => reject(err));
    pdf.on("end", () => resolve(Buffer.concat(chunks)));
  });
}
