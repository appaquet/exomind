
import Foundation
import JavaScriptCore

class EmailsLogic {
    static func createReplyEmail(_ entityTrait: EntityTraitOld) -> Command? {
        if  let entityJs = BridgeEntityConverter.entityToJavascript(entityTrait.entity),
            let email = entityTrait.trait as? EmailFull,
            let emailJs = BridgeEntityConverter.recordToJsRecord(email) {
        
            let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.createReplyEmail")
            let cmdObj = builderFunc?.call(withArguments: [entityJs, emailJs])
            return Command(jsObj: cmdObj!)
        } else {
            return nil
        }
    }
    
    static func createReplyAllEmail(_ entityTrait: EntityTraitOld) -> Command? {
        if  let entityJs = BridgeEntityConverter.entityToJavascript(entityTrait.entity),
            let email = entityTrait.trait as? EmailFull,
            let emailJs = BridgeEntityConverter.recordToJsRecord(email) {
            
            let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.createReplyAllEmail")
            let cmdObj = builderFunc?.call(withArguments: [entityJs, emailJs])
            return Command(jsObj: cmdObj!)
        } else {
            return nil
        }
    }

    static func createForwardEmail(_ entityTrait: EntityTraitOld) -> Command? {
        if  let entityJs = BridgeEntityConverter.entityToJavascript(entityTrait.entity),
            let email = entityTrait.trait as? EmailFull,
            let emailJs = BridgeEntityConverter.recordToJsRecord(email) {
            
            let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.createForwardEmail")
            let cmdObj = builderFunc?.call(withArguments: [entityJs, emailJs])
            return Command(jsObj: cmdObj!)
        } else {
            return nil
        }
    }

    static func sanitizeHtml(_ html: String) -> String {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.sanitizeHtml")
        return builderFunc!.call(withArguments: [html]).toString()
    }

    static func plainTextToHtml(_ text: String) -> String {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.plainTextToHtml")
        return builderFunc!.call(withArguments: [text]).toString()
    }

    static func splitOriginalThreadHtml(_ html: String) -> (String, String) {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.splitOriginalThreadHtml")
        let rets = builderFunc?.call(withArguments: [html]).toArray()
        return (rets![0] as! String, rets![1] as! String)
    }

    static func formatContact(_ contact: Contact, html: Bool = false, showAddress: Bool = false) -> String {
        if  let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.formatContact"),
            let jsContact = BridgeEntityConverter.recordToJsRecord(contact) {
            return builderFunc.call(withArguments: [jsContact, html, showAddress]).toString()
        } else {
            return contact.email
        }
    }

    static func formatContact(_ contact: Exomind_Base_Contact) -> String {
        if contact.name != "" {
            return "\(contact.name) <\(contact.email)>"
        } else {
            return contact.email
        }
    }

    static func injectInlineImages(_ entityTrait: EntityTraitOld, html: String) -> String {
        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.injectInlineImages")
        if  let jsEntity = BridgeEntityConverter.entityToJavascript(entityTrait.entity),
            let jsEmail = BridgeEntityConverter.recordToJsRecord(entityTrait.trait) {
        
            return builderFunc!.call(withArguments: [jsEntity, jsEmail, html]).toString()
        } else {
            return html
        }
    }

    static func attachmentUrl(_ entity: HCEntity, email: Email, attachment: FileAttachmentIntegration) -> String? {
        if  let jsEntity = BridgeEntityConverter.entityToJavascript(entity),
            let jsEmail = BridgeEntityConverter.recordToJsRecord(email),
            let jsAttachment = BridgeEntityConverter.recordToJsRecord(attachment),
            let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailsLogic.attachmentUrl") {

            return builderFunc.call(withArguments: [jsEntity, jsEmail, jsAttachment]).toString()
        } else {
            return nil
        }
    }

}

