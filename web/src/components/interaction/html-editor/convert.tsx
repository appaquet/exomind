import { convertFromHTML, convertToHTML } from "draft-convert";
import { ContentState } from "draft-js";
import React from "react";

export function convertOldHTML(html: string | undefined): string {
  if (!html) {
    return html;
  }

  // Squire added usless new lines after list items
  return html.replace(/<br>\s*<\/li>/mgi, "</li>");
}

export function toHTML(content: ContentState): string {
  return convertToHTML({
    // https://github.com/HubSpot/draft-convert
    blockToHTML: (block) => {
      // types are incorrect
      const tBlock = block as unknown as { type: string };
      if (tBlock.type === 'code-block') {
        return <code />;
      }
    },
    entityToHTML: (entity, originalText) => {
      if (entity.type === 'LINK') {
        return <a href={entity.data.url}>{originalText}</a>;
      }
      return originalText;
    }
  })(content);
}

export function fromHTML(html: string): ContentState {
  return convertFromHTML({
    // https://github.com/HubSpot/draft-convert
    htmlToBlock: (nodeName) => {
      if (nodeName === 'code') {
        return 'code-block';
      }
    },
    htmlToEntity: (nodeName, node, createEntity) => {
      const linkNode = node as HTMLLinkElement;
      if (nodeName === 'a') {
        return createEntity(
          'LINK',
          'MUTABLE',
          { url: linkNode.href }
        )
      }
    },
  })(html);
}