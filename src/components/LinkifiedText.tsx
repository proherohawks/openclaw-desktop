import { openUrl } from "../lib/shell";

/**
 * Regex to match URLs in plain text.
 * Matches http://, https://, and ftp:// URLs.
 * Avoids trailing punctuation (.,;:!?) unless inside parens.
 */
const URL_REGEX = /\bhttps?:\/\/[^\s<>[\]"'`]+[^\s<>[\]"'`.,;:!?)]/gi;

interface Props {
  text: string;
}

/**
 * Renders plain text with detected URLs as clickable links.
 * Used for user and error messages (which don't go through markdown).
 */
export default function LinkifiedText({ text }: Props) {
  const parts: (string | { url: string; key: number })[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let keyCounter = 0;

  // Reset regex state
  URL_REGEX.lastIndex = 0;

  while ((match = URL_REGEX.exec(text)) !== null) {
    // Add text before the URL
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    parts.push({ url: match[0], key: keyCounter++ });
    lastIndex = match.index + match[0].length;
  }

  // Add remaining text
  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  // If no URLs found, return plain text
  if (parts.length === 1 && typeof parts[0] === "string") {
    return <>{text}</>;
  }

  return (
    <>
      {parts.map((part) =>
        typeof part === "string" ? (
          part
        ) : (
          <a
            key={part.key}
            href={part.url}
            onClick={(e) => {
              e.preventDefault();
              openUrl(part.url);
            }}
            className="cursor-pointer text-claw-amber underline underline-offset-2 hover:text-claw-amber-light"
          >
            {part.url}
          </a>
        )
      )}
    </>
  );
}
