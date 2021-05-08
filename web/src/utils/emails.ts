/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/explicit-module-boundary-types */

import { Element, Node } from 'domhandler';
import * as domutils from 'domutils';
import { exomind } from '../protos';
import * as htmlparser from 'htmlparser2';
import domSerializerRender from "dom-serializer";
import linkify from 'linkifyjs';
import linkifyHtml from 'linkifyjs/html';
import _ from 'lodash';
import sanitizeHtml from 'sanitize-html';
import { EntityTrait, EntityTraits } from '../utils/entities';
import DateUtil from './date';
import { fromProtoTimestamp } from 'exocore';

export default class EmailUtil {
  static createReplyEmail(entity: EntityTraits, email: EntityTrait<exomind.base.IEmail>) {
    // TODO: Create reply email
    // let parts = EmailsLogicXYZ.generateReplyParts(email);
    // let draft = new Exomind.DraftEmail({
    //   to: [email.from],
    //   cc: [],
    //   bcc: [],
    //   attachments: [],
    //   from: email.source,
    //   subject: email.subject.get(),
    //   in_reply_to: email,
    //   parts: parts
    // });

    // return ExomindDSL.on(entity).mutate.put(draft).execute();
  }

  static createReplyAllEmail(entity: EntityTraits, email: EntityTrait<exomind.base.IEmail>) {
    // TODO: Create reply all email
    // let parts = EmailsLogic.generateReplyParts(email);
    // let draft = new Exomind.DraftEmail({
    //   to: [email.from],
    //   cc: _.flatten([email.to, email.cc]),
    //   bcc: [],
    //   attachments: [],
    //   from: email.source,
    //   subject: email.subject.get(),
    //   in_reply_to: email,
    //   parts: parts
    // });

    // return ExomindDSL.on(entity).mutate.put(draft).execute();
  }

  static createForwardEmail(entity: EntityTraits, email: EntityTrait<exomind.base.IEmail>) {
    // TODO: Create forward email
    // let parts = EmailsLogic.generateReplyParts(email);
    // let draft = new Exomind.DraftEmail({
    //   to: [],
    //   cc: [],
    //   bcc: [],
    //   attachments: [],
    //   from: email.source,
    //   subject: email.subject.get(),
    //   in_reply_to: email,
    //   parts: parts
    // });

    // return ExomindDSL.on(entity).mutate.put(draft).execute();
  }

  static generateReplyParts(entity: EntityTraits, email: EntityTrait<exomind.base.IEmail>) {
    const formattedReceiveDate = DateUtil.toLongGmtFormat(fromProtoTimestamp(email.message.receivedDate));
    const htmlPart = EmailUtil.extractHtmlPart(email.message.parts);
    const formattedFrom = EmailUtil.formatContact(email.message.from, true);
    const dateLine = `On ${formattedReceiveDate} ${formattedFrom} wrote:`;

    let parts: exomind.base.IEmailPart[] = [];
    if (htmlPart) {
      const part = new exomind.base.EmailPart(htmlPart);
      const html = EmailUtil.sanitizeHtml(part.body);
      const newPart = new exomind.base.EmailPart({
        body: `<br/><br/><div class="gmail_extra">${dateLine}<br/><blockquote style="margin:0 0 0 .8ex;border-left:1px #ccc solid;padding-left:1ex;font-size:1em">${html}</blockquote></div>`,
        mimeType: "text/html",
      })

      parts = [newPart];

    } else if (email.message.parts.length > 0) {
      const plainPart = _.first(email.message.parts);
      const body = EmailUtil.plainTextToHtml(plainPart.body);
      const newPart = new exomind.base.EmailPart({
        body: `<br/><br/><div class="gmail_extra">${dateLine}<br/><blockquote style="margin:0 0 0 .8ex;border-left:1px #ccc solid;padding-left:1ex;font-size:1em">${body}</blockquote></div>`,
        mimeType: "text/html",
      });
      parts = [newPart];
    }

    return parts;
  }

  static extractHtmlPart(parts: exomind.base.IEmailPart[]) {
    return _.find(parts, part => part.mimeType === 'text/html');
  }

  static parseContacts(contactsString: string): exomind.base.IContact[] {
    const len = contactsString.length;
    const contacts: exomind.base.IContact[] = [];

    let currentName = '';
    let currentEmail = ''
    let inEmailBracket = false;

    function addCumul() {
      currentName = currentName.trim();
      currentEmail = currentEmail.trim();
      if (currentName != '' && currentEmail === '') {
        currentEmail = currentName;
        currentName = '';
      }

      if (currentEmail !== '') {
        const contact = new exomind.base.Contact({ email: currentEmail, name: currentName });
        contacts.push(contact);
      }
      currentEmail = '';
      currentName = '';
      inEmailBracket = false;
    }

    for (let i = 0; i < len; i++) {
      const char = contactsString[i];
      if (char === ',' && !inEmailBracket) {
        addCumul();
      } else if (char === '<') {
        inEmailBracket = true;
      } else if (char === '>') {
        addCumul();
      } else if (!inEmailBracket) {
        currentName += char;
      } else if (inEmailBracket) {
        currentEmail += char;
      }
    }
    addCumul();

    return contacts;
  }

  static formatContact(contact: exomind.base.IContact, html = false, showAddress = false) {
    if (contact.name != '') {
      let ret = contact.name;
      if (showAddress) {
        if (!html) {
          ret += ` <${contact.email}>`;
        } else {
          ret += ` &lt;${contact.email}&gt;`;
        }
      }

      return ret;

    } else {
      return contact.email;
    }
  }

  static formatContacts(contacts: exomind.base.IContact[], showAddress = false) {
    return contacts.map((contact) => EmailUtil.formatContact(contact, false, showAddress)).join(', ');
  }

  static formatDate(date: Date) {
    return DateUtil.toShortFormat(new Date(date));
  }

  static plainTextToHtml(text: string) {
    const _ = linkify;
    return linkifyHtml(text.replace(/\n/g, '</br>'), {
      defaultProtocol: 'https',
    });
  }

  static sanitizeHtml(html: string) {
    // see https://www.npmjs.com/package/sanitize-html for defaults which are pretty good
    return sanitizeHtml(html, {
      allowedTags: sanitizeHtml.defaults.allowedTags.concat(['img', 'span', 'center', 'h1', 'h2', 'h3']),
      allowedAttributes: _.extend(sanitizeHtml.defaults.allowedAttributes, {
        'a': ['style', 'href', 'target'],
        '*': ['style', 'width', 'height', 'border', 'align', 'cellpadding', 'cellspacing', 'offset', 'valign', 'bgcolor', 'rowspan', 'colspan', 'background']
      }),
      nonTextTags: ['head', 'style', 'script', 'textarea', 'noscript', 'title'], // tags are are considered non-text, therefor removed. Added title to remove it
      transformTags: {
        'a': (tagName, attribs) => {
          return {
            tagName: 'a',
            attribs: _.extend(attribs, {
              target: '_blank'
            })
          };
        },
        'img': (tagName, attribs) => {
          // force http images to https
          if (attribs['src'] && attribs['src'].startsWith('http:')) {
            attribs['src'] = attribs['src'].replace('http:', 'https:');
          }

          return {
            tagName: 'img',
            attribs: attribs,
          };
        }
      }
    });
  }

  static injectInlineImages(entity: EntityTraits, email: EntityTrait<exomind.base.IEmail>, html: string) {
    _(email.message.attachments).filter(attach => !_.isEmpty(attach.inlinePlaceholder)).map(attach => {
      html = html.replace('cid:' + attach.inlinePlaceholder, EmailUtil.attachmentUrl(entity, email, attach));
    }).value();
    return html;
  }

  static attachmentUrl(entity: EntityTraits, email: EntityTrait<exomind.base.IEmail>, attachment: exomind.base.IEmailAttachment) {
    //return `${Constants.apiUrl}/files/attachments/?entityId=${entity.id}&traitId=${email.id}&key=${attachment.key}`;
    return 'http://exomind.io';
  }

  static splitOriginalThreadHtml(html: string) {
    const dom = htmlparser.parseDOM(html);

    function isOnWroteText(el: Node) {
      const text = domutils.getText(el).trim();

      const length = text.length;
      if (length < 200) {
        const lines = text.split('\n');
        const hasWrote = (text.lastIndexOf('wrote') / length) > 0.7;
        const hasEcrit = (text.lastIndexOf('crit') / length) > 0.7;
        const lastCollon = (text.lastIndexOf(':') / length) > 0.9;

        return lines.length < 2 && (hasWrote || hasEcrit) && lastCollon;
      } else {
        return false;
      }
    }

    function parseRecursive(el: Element | Element[], nextIsOriginal = false) {
      if (_.isArray(el)) {
        let currents: Element[] = [];
        let originals: Element[] = [];
        el.reverse().forEach(child => {
          const [current, original] = parseRecursive(child, nextIsOriginal);
          currents = current.concat(currents);
          originals = original.concat(originals);
        });
        return [currents, originals];

      } else {
        if (domutils.getAttributeValue(el, 'class') === 'gmail_quote' || domutils.getAttributeValue(el, 'class') === 'gmail_extra') {
          return [[], [el]];

        } else if (el.type === 'tag' && el.name === 'blockquote') {
          return [[], [el]];

        } else if (isOnWroteText(el)) {
          return [[], [el]];

        } else if (!_.isEmpty(el.children)) {
          const [currentChildren, originalChildren] = parseRecursive(el.children as Element[], nextIsOriginal);
          const currentEl = _.clone(el);
          currentEl.children = currentChildren;
          const originalEl = _.clone(el);
          originalEl.children = originalChildren;
          return [[currentEl], [originalEl]];
        } else {
          return [[el], []]
        }
      }
    }

    // eslint-disable-next-line prefer-const
    let [current, original] = parseRecursive(dom as Element[]);

    if (domutils.getText(current).trim().length === 0) {
      current = original;
      return [domSerializerRender(current), ''];
    } else {
      return [domSerializerRender(current), domSerializerRender(original)];
    }
  }
}