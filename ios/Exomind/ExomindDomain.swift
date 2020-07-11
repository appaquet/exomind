
import Foundation


class ExomindNamespace: HCNamespace {
  let name: String = "exomind"
  func builderForType(_ fullType: String) -> HCRecordBuilder? {
    switch fullType {
      
      case "exomind.link":
        return LinkBuilder()
             
      
      case "exomind.email_part_html":
        return EmailPartHtmlBuilder()
             
      
      case "exomind.integration":
        return IntegrationBuilder()
             
      
      case "exomind.email_thread":
        return EmailThreadBuilder()
             
      
      case "exomind.draft_email":
        return DraftEmailBuilder()
             
      
      case "exomind.child":
        return ChildBuilder()
             
      
      case "exomind.special":
        return SpecialBuilder()
             
      
      case "exomind.contact":
        return ContactBuilder()
             
      
      case "exomind.collection":
        return CollectionBuilder()
             
      
      case "exomind.file_attachment_integration":
        return FileAttachmentIntegrationBuilder()
             
      
      case "exomind.task":
        return TaskBuilder()
             
      
      case "exomind.lineage":
        return LineageBuilder()
             
      
      case "exomind.note":
        return NoteBuilder()
             
      
      case "exomind.email":
        return EmailBuilder()
             
      
      case "exomind.integration_source":
        return IntegrationSourceBuilder()
             
      
      case "exomind.old_child":
        return OldChildBuilder()
             
      
      case "exomind.postponed":
        return PostponedBuilder()
             
      
      case "exomind.email_part_plain":
        return EmailPartPlainBuilder()
             
      default:
        return nil
     }
  }
}
    

class EmailThreadSchema: HCRecordSchema {
  static let type = "email_thread"
  static let fullType = "exomind.email_thread"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let fromField = HCField(name: "from", type: HCStructureFieldType(name: "contact"))
  static let snippetField = HCField(name: "snippet", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let sourceField = HCField(name: "source", type: HCStructureFieldType(name: "integration_source"))
  static let lastEmailField = HCField(name: "last_email", type: HCOptionFieldType(subtype: HCTraitReferenceFieldType()))
  static let subjectField = HCField(name: "subject", type: HCOptionFieldType(subtype: HCStringFieldType()))
  let fields: [HCField] = [EmailThreadSchema.creationDateField, EmailThreadSchema.lastEmailField, EmailThreadSchema.subjectField, EmailThreadSchema.fromField, EmailThreadSchema.modificationDateField, EmailThreadSchema.snippetField, EmailThreadSchema.sourceField]
}
     

class LineageSchema: HCRecordSchema {
  static let type = "lineage"
  static let fullType = "exomind.lineage"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let processedDateField = HCField(name: "processed_date", type: HCDateFieldType())
  static let depthField = HCField(name: "depth", type: HCLongFieldType())
  static let toField = HCField(name: "to", type: HCEntityReferenceFieldType())
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let parentNameField = HCField(name: "parent_name", type: HCOptionFieldType(subtype: HCStringFieldType()))
  let fields: [HCField] = [LineageSchema.creationDateField, LineageSchema.toField, LineageSchema.processedDateField, LineageSchema.modificationDateField, LineageSchema.depthField, LineageSchema.parentNameField]
}
     

class ChildSchema: HCRecordSchema {
  static let type = "child"
  static let fullType = "exomind.child"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let toField = HCField(name: "to", type: HCEntityReferenceFieldType())
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let weightField = HCField(name: "weight", type: HCLongFieldType())
  static let dateField = HCField(name: "date", type: HCDateFieldType())
  let fields: [HCField] = [ChildSchema.weightField, ChildSchema.modificationDateField, ChildSchema.toField, ChildSchema.dateField, ChildSchema.creationDateField]
}
     

class DraftEmailSchema: HCRecordSchema {
  static let type = "draft_email"
  static let fullType = "exomind.draft_email"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let bccField = HCField(name: "bcc", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")))
  static let partsField = HCField(name: "parts", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "email_part")))
  static let sendingDateField = HCField(name: "sending_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let attachmentsField = HCField(name: "attachments", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "file_attachment")))
  static let toField = HCField(name: "to", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")))
  static let inReplyToField = HCField(name: "in_reply_to", type: HCOptionFieldType(subtype: HCTraitReferenceFieldType()))
  static let fromField = HCField(name: "from", type: HCOptionFieldType(subtype: HCStructureFieldType(name: "integration_source")))
  static let sentDateField = HCField(name: "sent_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let subjectField = HCField(name: "subject", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let ccField = HCField(name: "cc", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")))
  let fields: [HCField] = [DraftEmailSchema.ccField, DraftEmailSchema.toField, DraftEmailSchema.fromField, DraftEmailSchema.sendingDateField, DraftEmailSchema.partsField, DraftEmailSchema.attachmentsField, DraftEmailSchema.subjectField, DraftEmailSchema.bccField, DraftEmailSchema.creationDateField, DraftEmailSchema.sentDateField, DraftEmailSchema.modificationDateField, DraftEmailSchema.inReplyToField]
}
     

class EmailPartHtmlSchema: HCRecordSchema {
  static let type = "email_part_html"
  static let fullType = "exomind.email_part_html"
  static let bodyField = HCField(name: "body", type: HCStringFieldType())
  let fields: [HCField] = [EmailPartHtmlSchema.bodyField]
}
     

class IntegrationSchema: HCRecordSchema {
  static let type = "integration"
  static let fullType = "exomind.integration"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let keyField = HCField(name: "key", type: HCStringFieldType())
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let dataField = HCField(name: "data", type: HCMapFieldType(subtype: HCStringFieldType()))
  static let nameField = HCField(name: "name", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let typField = HCField(name: "typ", type: HCStringFieldType())
  let fields: [HCField] = [IntegrationSchema.nameField, IntegrationSchema.typField, IntegrationSchema.dataField, IntegrationSchema.modificationDateField, IntegrationSchema.keyField, IntegrationSchema.creationDateField]
}
     

class PostponedSchema: HCRecordSchema {
  static let type = "postponed"
  static let fullType = "exomind.postponed"
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let untilDateField = HCField(name: "until_date", type: HCDateFieldType())
  let fields: [HCField] = [PostponedSchema.creationDateField, PostponedSchema.modificationDateField, PostponedSchema.untilDateField]
}
     

class EmailSchema: HCRecordSchema {
  static let type = "email"
  static let fullType = "exomind.email"
  static let unreadField = HCField(name: "unread", type: HCOptionFieldType(subtype: HCBooleanFieldType()))
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let idField = HCField(name: "id", type: HCStringFieldType())
  static let bccField = HCField(name: "bcc", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")))
  static let partsField = HCField(name: "parts", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "email_part")))
  static let fromField = HCField(name: "from", type: HCStructureFieldType(name: "contact"))
  static let snippetField = HCField(name: "snippet", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let attachmentsField = HCField(name: "attachments", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "file_attachment")))
  static let sourceField = HCField(name: "source", type: HCStructureFieldType(name: "integration_source"))
  static let toField = HCField(name: "to", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")))
  static let receivedDateField = HCField(name: "received_date", type: HCDateFieldType())
  static let subjectField = HCField(name: "subject", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let ccField = HCField(name: "cc", type: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")))
  let fields: [HCField] = [EmailSchema.toField, EmailSchema.fromField, EmailSchema.sourceField, EmailSchema.partsField, EmailSchema.snippetField, EmailSchema.subjectField, EmailSchema.bccField, EmailSchema.attachmentsField, EmailSchema.unreadField, EmailSchema.receivedDateField, EmailSchema.ccField, EmailSchema.idField, EmailSchema.creationDateField, EmailSchema.modificationDateField]
}
     

class ContactSchema: HCRecordSchema {
  static let type = "contact"
  static let fullType = "exomind.contact"
  static let emailField = HCField(name: "email", type: HCStringFieldType())
  static let nameField = HCField(name: "name", type: HCOptionFieldType(subtype: HCStringFieldType()))
  let fields: [HCField] = [ContactSchema.emailField, ContactSchema.nameField]
}
     

class IntegrationSourceSchema: HCRecordSchema {
  static let type = "integration_source"
  static let fullType = "exomind.integration_source"
  static let integrationNameField = HCField(name: "integration_name", type: HCStringFieldType())
  static let integrationKeyField = HCField(name: "integration_key", type: HCStringFieldType())
  static let dataField = HCField(name: "data", type: HCMapFieldType(subtype: HCStringFieldType()))
  let fields: [HCField] = [IntegrationSourceSchema.integrationNameField, IntegrationSourceSchema.integrationKeyField, IntegrationSourceSchema.dataField]
}
     

class EmailPartSchema: HCRecordSchema {
  static let type = "email_part"
  static let fullType = "exomind.email_part"
  static let bodyField = HCField(name: "body", type: HCStringFieldType())
  let fields: [HCField] = [EmailPartSchema.bodyField]
}
     

class LinkSchema: HCRecordSchema {
  static let type = "link"
  static let fullType = "exomind.link"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let idField = HCField(name: "id", type: HCStringFieldType())
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let titleField = HCField(name: "title", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let urlField = HCField(name: "url", type: HCStringFieldType())
  let fields: [HCField] = [LinkSchema.modificationDateField, LinkSchema.urlField, LinkSchema.titleField, LinkSchema.creationDateField, LinkSchema.idField]
}
     

class NoteSchema: HCRecordSchema {
  static let type = "note"
  static let fullType = "exomind.note"
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let idField = HCField(name: "id", type: HCStringFieldType())
  static let contentField = HCField(name: "content", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let titleField = HCField(name: "title", type: HCStringFieldType())
  let fields: [HCField] = [NoteSchema.creationDateField, NoteSchema.contentField, NoteSchema.idField, NoteSchema.modificationDateField, NoteSchema.titleField]
}
     

class OldChildSchema: HCRecordSchema {
  static let type = "old_child"
  static let fullType = "exomind.old_child"
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let toField = HCField(name: "to", type: HCEntityReferenceFieldType())
  static let dateField = HCField(name: "date", type: HCDateFieldType())
  let fields: [HCField] = [OldChildSchema.creationDateField, OldChildSchema.modificationDateField, OldChildSchema.toField, OldChildSchema.dateField]
}
     

class EmailPartPlainSchema: HCRecordSchema {
  static let type = "email_part_plain"
  static let fullType = "exomind.email_part_plain"
  static let bodyField = HCField(name: "body", type: HCStringFieldType())
  let fields: [HCField] = [EmailPartPlainSchema.bodyField]
}
     

class FileAttachmentSchema: HCRecordSchema {
  static let type = "file_attachment"
  static let fullType = "exomind.file_attachment"
  static let nameField = HCField(name: "name", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let mimeField = HCField(name: "mime", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let sizeField = HCField(name: "size", type: HCOptionFieldType(subtype: HCLongFieldType()))
  static let inlinePlaceholderField = HCField(name: "inline_placeholder", type: HCOptionFieldType(subtype: HCStringFieldType()))
  let fields: [HCField] = [FileAttachmentSchema.nameField, FileAttachmentSchema.mimeField, FileAttachmentSchema.sizeField, FileAttachmentSchema.inlinePlaceholderField]
}
     

class CollectionSchema: HCRecordSchema {
  static let type = "collection"
  static let fullType = "exomind.collection"
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let nameField = HCField(name: "name", type: HCStringFieldType())
  static let iconField = HCField(name: "icon", type: HCOptionFieldType(subtype: HCStringFieldType()))
  let fields: [HCField] = [CollectionSchema.creationDateField, CollectionSchema.modificationDateField, CollectionSchema.nameField, CollectionSchema.iconField]
}
     

class TaskSchema: HCRecordSchema {
  static let type = "task"
  static let fullType = "exomind.task"
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let idField = HCField(name: "id", type: HCStringFieldType())
  static let titleField = HCField(name: "title", type: HCStringFieldType())
  let fields: [HCField] = [TaskSchema.creationDateField, TaskSchema.modificationDateField, TaskSchema.idField, TaskSchema.titleField]
}
     

class SpecialSchema: HCRecordSchema {
  static let type = "special"
  static let fullType = "exomind.special"
  static let creationDateField = HCField(name: "creation_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let modificationDateField = HCField(name: "modification_date", type: HCOptionFieldType(subtype: HCDateFieldType()))
  static let nameField = HCField(name: "name", type: HCStringFieldType())
  let fields: [HCField] = [SpecialSchema.creationDateField, SpecialSchema.modificationDateField, SpecialSchema.nameField]
}
     

class FileAttachmentIntegrationSchema: HCRecordSchema {
  static let type = "file_attachment_integration"
  static let fullType = "exomind.file_attachment_integration"
  static let mimeField = HCField(name: "mime", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let keyField = HCField(name: "key", type: HCStringFieldType())
  static let inlinePlaceholderField = HCField(name: "inline_placeholder", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let dataField = HCField(name: "data", type: HCMapFieldType(subtype: HCStringFieldType()))
  static let nameField = HCField(name: "name", type: HCOptionFieldType(subtype: HCStringFieldType()))
  static let integrationNameField = HCField(name: "integration_name", type: HCStringFieldType())
  static let integrationKeyField = HCField(name: "integration_key", type: HCStringFieldType())
  static let sizeField = HCField(name: "size", type: HCOptionFieldType(subtype: HCLongFieldType()))
  let fields: [HCField] = [FileAttachmentIntegrationSchema.mimeField, FileAttachmentIntegrationSchema.sizeField, FileAttachmentIntegrationSchema.inlinePlaceholderField, FileAttachmentIntegrationSchema.integrationKeyField, FileAttachmentIntegrationSchema.keyField, FileAttachmentIntegrationSchema.nameField, FileAttachmentIntegrationSchema.integrationNameField, FileAttachmentIntegrationSchema.dataField]
}
     

protocol FileAttachment: HCStructure {
  var name:String? { get }
}
     

protocol EmailPart: HCStructure {

}
     

protocol EmailPartHtml: HCStructure {
  
}

class EmailPartHtmlFull: EmailPartHtml, HCFullRecord, EmailPart {
  let type = "email_part_html"
  let fullType = "exomind.email_part_html"
  let schema: HCRecordSchema = EmailPartHtmlSchema()
  var traitId: HCTraitId?

  var body:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "body":
      return self.body as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(body: String) {
    self.body = body
    
  }
       
}

class EmailPartHtmlSummary: EmailPartHtml, HCSummaryRecord, EmailPart {
  let type = "email_part_html"
  let fullType = "exomind.email_part_html"
  let schema: HCRecordSchema = EmailPartHtmlSchema()
  var traitId: HCTraitId?

  
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init() {
    
  }
       
}

class EmailPartHtmlBuilder: HCStructureBuilder {
  let type = "email_part_html"
  let fullType = "exomind.email_part_html"
  var error: HCBuildingError?
  let schema: HCRecordSchema = EmailPartHtmlSchema()
  var traitId: HCTraitId?

  var body:String? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "body":
      return self.body as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(body: String? = nil) {
    
    if let value = body {
      self.body = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "body":
      
      if let casted = value as? String {
        self.body = casted
        return
               
      } else {
        error = .invalidFieldType(name: "body", expectedType: HCStringFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.body == nil {
      error = .missingField(name: "body")
      return nil
    }
           
    let record = EmailPartHtmlFull(body: self.body!)
    
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    let record = EmailPartHtmlSummary()
    
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol EmailThread: HCTrait {
  var source:IntegrationSource { get }
  var subject:String? { get }
  var from:Contact { get }
  var modificationDate:Date? { get }
  var snippet:String? { get }
  var lastEmail:HCTraitId? { get }
  var creationDate:Date? { get }
}

class EmailThreadFull: EmailThread, HCFullRecord {
  let type = "email_thread"
  let fullType = "exomind.email_thread"
  let schema: HCRecordSchema = EmailThreadSchema()
  var traitId: HCTraitId?

  var lastEmail:HCTraitId?
  var snippet:String?
  var modificationDate:Date?
  var source:IntegrationSource
  var creationDate:Date?
  var from:Contact
  var subject:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "from":
      return self.from as Any?
           
    case "last_email":
      return self.lastEmail as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "snippet":
      return self.snippet as Any?
           
    case "source":
      return self.source as Any?
           
    case "subject":
      return self.subject as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(from: Contact, source: IntegrationSource, creationDate: Date? = nil, lastEmail: HCTraitId? = nil, modificationDate: Date? = nil, snippet: String? = nil, subject: String? = nil) {
    self.from = from
    self.source = source
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = lastEmail {
      self.lastEmail = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = snippet {
      self.snippet = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
  }
       
}

class EmailThreadSummary: EmailThread, HCSummaryRecord {
  let type = "email_thread"
  let fullType = "exomind.email_thread"
  let schema: HCRecordSchema = EmailThreadSchema()
  var traitId: HCTraitId?

  var lastEmail:HCTraitId?
  var snippet:String?
  var modificationDate:Date?
  var source:IntegrationSource
  var creationDate:Date?
  var from:Contact
  var subject:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "from":
      return self.from as Any?
           
    case "last_email":
      return self.lastEmail as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "snippet":
      return self.snippet as Any?
           
    case "source":
      return self.source as Any?
           
    case "subject":
      return self.subject as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(from: Contact, source: IntegrationSource, creationDate: Date? = nil, lastEmail: HCTraitId? = nil, modificationDate: Date? = nil, snippet: String? = nil, subject: String? = nil) {
    self.from = from
    self.source = source
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = lastEmail {
      self.lastEmail = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = snippet {
      self.snippet = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
  }
       
}

class EmailThreadBuilder: HCTraitBuilder {
  let type = "email_thread"
  let fullType = "exomind.email_thread"
  var error: HCBuildingError?
  let schema: HCRecordSchema = EmailThreadSchema()
  var traitId: HCTraitId?

  var from:Contact? = nil
  var lastEmail:HCTraitId?? = nil
  var source:IntegrationSource? = nil
  var subject:String?? = nil
  var modificationDate:Date?? = nil
  var snippet:String?? = nil
  var creationDate:Date?? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "from":
      return self.from as Any??
           
    case "last_email":
      return self.lastEmail as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "snippet":
      return self.snippet as Any??
           
    case "source":
      return self.source as Any??
           
    case "subject":
      return self.subject as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(from: Contact? = nil, modificationDate: Date?? = nil, lastEmail: HCTraitId?? = nil, source: IntegrationSource? = nil, snippet: String?? = nil, subject: String?? = nil, creationDate: Date?? = nil) {
    
    if let value = subject {
      self.subject = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = lastEmail {
      self.lastEmail = value
    }
           
    if let value = snippet {
      self.snippet = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = from {
      self.from = value
    }
           
    if let value = source {
      self.source = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "last_email":
      
      if let casted = value as? HCTraitId {
        self.lastEmail = casted
        return
               
      } else {
        error = .invalidFieldType(name: "last_email", expectedType: HCOptionFieldType(subtype: HCTraitReferenceFieldType()), value: value)
      }
           
    case "from":
      
      if let casted = value as? Contact {
        self.from = casted
        return
               
      } else {
        error = .invalidFieldType(name: "from", expectedType: HCStructureFieldType(name: "contact"), value: value)
      }
           
    case "snippet":
      
      if let casted = value as? String {
        self.snippet = casted
        return
               
      } else {
        error = .invalidFieldType(name: "snippet", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "subject":
      
      if let casted = value as? String {
        self.subject = casted
        return
               
      } else {
        error = .invalidFieldType(name: "subject", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "source":
      
      if let casted = value as? IntegrationSource {
        self.source = casted
        return
               
      } else {
        error = .invalidFieldType(name: "source", expectedType: HCStructureFieldType(name: "integration_source"), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.from == nil {
      error = .missingField(name: "from")
      return nil
    }
           
    if self.source == nil {
      error = .missingField(name: "source")
      return nil
    }
           
    let record = EmailThreadFull(from: self.from!, source: self.source!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.lastEmail {
      record.lastEmail = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.snippet {
      record.snippet = value
    }
           
    if let value = self.subject {
      record.subject = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.from == nil {
      error = .missingField(name: "from")
      return nil
    }
           
    if self.source == nil {
      error = .missingField(name: "source")
      return nil
    }
           
    let record = EmailThreadSummary(from: self.from!, source: self.source!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.lastEmail {
      record.lastEmail = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.snippet {
      record.snippet = value
    }
           
    if let value = self.subject {
      record.subject = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Task: HCTrait {
  var creationDate:Date? { get }
  var modificationDate:Date? { get }
  var id:String { get }
  var title:String { get }
}

class TaskFull: Task, HCFullRecord {
  let type = "task"
  let fullType = "exomind.task"
  let schema: HCRecordSchema = TaskSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var id:String
  var title:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "title":
      return self.title as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(id: String, title: String, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.id = id
    self.title = title
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class TaskSummary: Task, HCSummaryRecord {
  let type = "task"
  let fullType = "exomind.task"
  let schema: HCRecordSchema = TaskSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var id:String
  var title:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "title":
      return self.title as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(id: String, title: String, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.id = id
    self.title = title
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class TaskBuilder: HCTraitBuilder {
  let type = "task"
  let fullType = "exomind.task"
  var error: HCBuildingError?
  let schema: HCRecordSchema = TaskSchema()
  var traitId: HCTraitId?

  var creationDate:Date?? = nil
  var modificationDate:Date?? = nil
  var id:String? = nil
  var title:String? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "id":
      return self.id as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "title":
      return self.title as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(creationDate: Date?? = nil, modificationDate: Date?? = nil, id: String? = nil, title: String? = nil) {
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = id {
      self.id = value
    }
           
    if let value = title {
      self.title = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "id":
      
      if let casted = value as? String {
        self.id = casted
        return
               
      } else {
        error = .invalidFieldType(name: "id", expectedType: HCStringFieldType(), value: value)
      }
           
    case "title":
      
      if let casted = value as? String {
        self.title = casted
        return
               
      } else {
        error = .invalidFieldType(name: "title", expectedType: HCStringFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.title == nil {
      error = .missingField(name: "title")
      return nil
    }
           
    let record = TaskFull(id: self.id!, title: self.title!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.title == nil {
      error = .missingField(name: "title")
      return nil
    }
           
    let record = TaskSummary(id: self.id!, title: self.title!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Postponed: HCTrait {
  var creationDate:Date? { get }
  var modificationDate:Date? { get }
  var untilDate:Date { get }
}

class PostponedFull: Postponed, HCFullRecord {
  let type = "postponed"
  let fullType = "exomind.postponed"
  let schema: HCRecordSchema = PostponedSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var untilDate:Date
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "until_date":
      return self.untilDate as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(untilDate: Date, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.untilDate = untilDate
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class PostponedSummary: Postponed, HCSummaryRecord {
  let type = "postponed"
  let fullType = "exomind.postponed"
  let schema: HCRecordSchema = PostponedSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var untilDate:Date
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "until_date":
      return self.untilDate as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(untilDate: Date, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.untilDate = untilDate
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class PostponedBuilder: HCTraitBuilder {
  let type = "postponed"
  let fullType = "exomind.postponed"
  var error: HCBuildingError?
  let schema: HCRecordSchema = PostponedSchema()
  var traitId: HCTraitId?

  var creationDate:Date?? = nil
  var modificationDate:Date?? = nil
  var untilDate:Date? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "until_date":
      return self.untilDate as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(creationDate: Date?? = nil, modificationDate: Date?? = nil, untilDate: Date? = nil) {
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = untilDate {
      self.untilDate = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "until_date":
      
      if let casted = value as? Date {
        self.untilDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "until_date", expectedType: HCDateFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.untilDate == nil {
      error = .missingField(name: "until_date")
      return nil
    }
           
    let record = PostponedFull(untilDate: self.untilDate!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.untilDate == nil {
      error = .missingField(name: "until_date")
      return nil
    }
           
    let record = PostponedSummary(untilDate: self.untilDate!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Collection: HCTrait {
  var creationDate:Date? { get }
  var modificationDate:Date? { get }
  var name:String { get }
  var icon:String? { get }
}

class CollectionFull: Collection, HCFullRecord {
  let type = "collection"
  let fullType = "exomind.collection"
  let schema: HCRecordSchema = CollectionSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var name:String
  var icon:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "icon":
      return self.icon as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(name: String, creationDate: Date? = nil, icon: String? = nil, modificationDate: Date? = nil) {
    self.name = name
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = icon {
      self.icon = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class CollectionSummary: Collection, HCSummaryRecord {
  let type = "collection"
  let fullType = "exomind.collection"
  let schema: HCRecordSchema = CollectionSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var name:String
  var icon:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "icon":
      return self.icon as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(name: String, creationDate: Date? = nil, icon: String? = nil, modificationDate: Date? = nil) {
    self.name = name
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = icon {
      self.icon = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class CollectionBuilder: HCTraitBuilder {
  let type = "collection"
  let fullType = "exomind.collection"
  var error: HCBuildingError?
  let schema: HCRecordSchema = CollectionSchema()
  var traitId: HCTraitId?

  var creationDate:Date?? = nil
  var modificationDate:Date?? = nil
  var name:String? = nil
  var icon:String?? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "icon":
      return self.icon as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "name":
      return self.name as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(creationDate: Date?? = nil, modificationDate: Date?? = nil, name: String? = nil, icon: String?? = nil) {
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = name {
      self.name = value
    }
           
    if let value = icon {
      self.icon = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "name":
      
      if let casted = value as? String {
        self.name = casted
        return
               
      } else {
        error = .invalidFieldType(name: "name", expectedType: HCStringFieldType(), value: value)
      }
           
    case "icon":
      
      if let casted = value as? String {
        self.icon = casted
        return
               
      } else {
        error = .invalidFieldType(name: "icon", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.name == nil {
      error = .missingField(name: "name")
      return nil
    }
           
    let record = CollectionFull(name: self.name!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.icon {
      record.icon = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.name == nil {
      error = .missingField(name: "name")
      return nil
    }
           
    let record = CollectionSummary(name: self.name!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.icon {
      record.icon = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Special: HCTrait {
  var creationDate:Date? { get }
  var modificationDate:Date? { get }
  var name:String { get }
}

class SpecialFull: Special, HCFullRecord {
  let type = "special"
  let fullType = "exomind.special"
  let schema: HCRecordSchema = SpecialSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var name:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(name: String, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.name = name
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class SpecialSummary: Special, HCSummaryRecord {
  let type = "special"
  let fullType = "exomind.special"
  let schema: HCRecordSchema = SpecialSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var name:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(name: String, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.name = name
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class SpecialBuilder: HCTraitBuilder {
  let type = "special"
  let fullType = "exomind.special"
  var error: HCBuildingError?
  let schema: HCRecordSchema = SpecialSchema()
  var traitId: HCTraitId?

  var creationDate:Date?? = nil
  var modificationDate:Date?? = nil
  var name:String? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "name":
      return self.name as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(creationDate: Date?? = nil, modificationDate: Date?? = nil, name: String? = nil) {
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = name {
      self.name = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "name":
      
      if let casted = value as? String {
        self.name = casted
        return
               
      } else {
        error = .invalidFieldType(name: "name", expectedType: HCStringFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.name == nil {
      error = .missingField(name: "name")
      return nil
    }
           
    let record = SpecialFull(name: self.name!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.name == nil {
      error = .missingField(name: "name")
      return nil
    }
           
    let record = SpecialSummary(name: self.name!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Contact: HCStructure {
  var email:String { get }
  var name:String? { get }
}

class ContactFull: Contact, HCFullRecord {
  let type = "contact"
  let fullType = "exomind.contact"
  let schema: HCRecordSchema = ContactSchema()
  var traitId: HCTraitId?

  var email:String
  var name:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "email":
      return self.email as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(email: String, name: String? = nil) {
    self.email = email
    
    if let value = name {
      self.name = value
    }
           
  }
       
}

class ContactSummary: Contact, HCSummaryRecord {
  let type = "contact"
  let fullType = "exomind.contact"
  let schema: HCRecordSchema = ContactSchema()
  var traitId: HCTraitId?

  var email:String
  var name:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "email":
      return self.email as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(email: String, name: String? = nil) {
    self.email = email
    
    if let value = name {
      self.name = value
    }
           
  }
       
}

class ContactBuilder: HCStructureBuilder {
  let type = "contact"
  let fullType = "exomind.contact"
  var error: HCBuildingError?
  let schema: HCRecordSchema = ContactSchema()
  var traitId: HCTraitId?

  var email:String? = nil
  var name:String?? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "email":
      return self.email as Any??
           
    case "name":
      return self.name as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(email: String? = nil, name: String?? = nil) {
    
    if let value = email {
      self.email = value
    }
           
    if let value = name {
      self.name = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "email":
      
      if let casted = value as? String {
        self.email = casted
        return
               
      } else {
        error = .invalidFieldType(name: "email", expectedType: HCStringFieldType(), value: value)
      }
           
    case "name":
      
      if let casted = value as? String {
        self.name = casted
        return
               
      } else {
        error = .invalidFieldType(name: "name", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.email == nil {
      error = .missingField(name: "email")
      return nil
    }
           
    let record = ContactFull(email: self.email!)
    
    if let value = self.name {
      record.name = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.email == nil {
      error = .missingField(name: "email")
      return nil
    }
           
    let record = ContactSummary(email: self.email!)
    
    if let value = self.name {
      record.name = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol IntegrationSource: HCStructure {
  var integrationName:String { get }
  var integrationKey:String { get }
}

class IntegrationSourceFull: IntegrationSource, HCFullRecord {
  let type = "integration_source"
  let fullType = "exomind.integration_source"
  let schema: HCRecordSchema = IntegrationSourceSchema()
  var traitId: HCTraitId?

  var integrationName:String
  var integrationKey:String
  var data:[String : String]
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "data":
      return self.data as Any?
           
    case "integration_key":
      return self.integrationKey as Any?
           
    case "integration_name":
      return self.integrationName as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    case "data":
      return self.data.mapPairs { (tup) in (tup.0, tup.1 as Any) }
             
    default:
      return [:]
    }
  }
       
  
  init(data: [String : String], integrationKey: String, integrationName: String) {
    self.data = data
    self.integrationKey = integrationKey
    self.integrationName = integrationName
    
  }
       
}

class IntegrationSourceSummary: IntegrationSource, HCSummaryRecord {
  let type = "integration_source"
  let fullType = "exomind.integration_source"
  let schema: HCRecordSchema = IntegrationSourceSchema()
  var traitId: HCTraitId?

  var integrationName:String
  var integrationKey:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "integration_key":
      return self.integrationKey as Any?
           
    case "integration_name":
      return self.integrationName as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(integrationKey: String, integrationName: String) {
    self.integrationKey = integrationKey
    self.integrationName = integrationName
    
  }
       
}

class IntegrationSourceBuilder: HCStructureBuilder {
  let type = "integration_source"
  let fullType = "exomind.integration_source"
  var error: HCBuildingError?
  let schema: HCRecordSchema = IntegrationSourceSchema()
  var traitId: HCTraitId?

  var integrationName:String? = nil
  var integrationKey:String? = nil
  var data:[String : String]? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "data":
      return self.data as Any??
           
    case "integration_key":
      return self.integrationKey as Any??
           
    case "integration_name":
      return self.integrationName as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    case "data":
      return self.data.map { $0.mapPairs { (tup) in (tup.0, tup.1 as Any) } }
             
    default:
      return [:]
    }
  }
       
  
  init(integrationName: String? = nil, integrationKey: String? = nil, data: [String : String]? = nil) {
    
    if let value = integrationName {
      self.integrationName = value
    }
           
    if let value = integrationKey {
      self.integrationKey = value
    }
           
    if let value = data {
      self.data = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "integration_name":
      
      if let casted = value as? String {
        self.integrationName = casted
        return
               
      } else {
        error = .invalidFieldType(name: "integration_name", expectedType: HCStringFieldType(), value: value)
      }
           
    case "integration_key":
      
      if let casted = value as? String {
        self.integrationKey = casted
        return
               
      } else {
        error = .invalidFieldType(name: "integration_key", expectedType: HCStringFieldType(), value: value)
      }
           
    case "data":
      
      if let arr = value as? [String : Any] {
        var casted = [ String : String ]()
        arr.forEach({ (tup) in
          if let nv = tup.1 as? String {
            casted[tup.0] = nv
          }
        })
        self.data = casted
        return
               
      } else {
        error = .invalidFieldType(name: "data", expectedType: HCMapFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.data == nil {
      error = .missingField(name: "data")
      return nil
    }
           
    if self.integrationKey == nil {
      error = .missingField(name: "integration_key")
      return nil
    }
           
    if self.integrationName == nil {
      error = .missingField(name: "integration_name")
      return nil
    }
           
    let record = IntegrationSourceFull(data: self.data!, integrationKey: self.integrationKey!, integrationName: self.integrationName!)
    
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.integrationKey == nil {
      error = .missingField(name: "integration_key")
      return nil
    }
           
    if self.integrationName == nil {
      error = .missingField(name: "integration_name")
      return nil
    }
           
    let record = IntegrationSourceSummary(integrationKey: self.integrationKey!, integrationName: self.integrationName!)
    
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Integration: HCTrait {
  var typ:String { get }
  var key:String { get }
  var name:String? { get }
  var modificationDate:Date? { get }
  var creationDate:Date? { get }
}

class IntegrationFull: Integration, HCFullRecord {
  let type = "integration"
  let fullType = "exomind.integration"
  let schema: HCRecordSchema = IntegrationSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var typ:String
  var name:String?
  var data:[String : String]
  var creationDate:Date?
  var key:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "data":
      return self.data as Any?
           
    case "key":
      return self.key as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "name":
      return self.name as Any?
           
    case "typ":
      return self.typ as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    case "data":
      return self.data.mapPairs { (tup) in (tup.0, tup.1 as Any) }
             
    default:
      return [:]
    }
  }
       
  
  init(data: [String : String], key: String, typ: String, creationDate: Date? = nil, modificationDate: Date? = nil, name: String? = nil) {
    self.data = data
    self.key = key
    self.typ = typ
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = name {
      self.name = value
    }
           
  }
       
}

class IntegrationSummary: Integration, HCSummaryRecord {
  let type = "integration"
  let fullType = "exomind.integration"
  let schema: HCRecordSchema = IntegrationSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var typ:String
  var name:String?
  var creationDate:Date?
  var key:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "key":
      return self.key as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "name":
      return self.name as Any?
           
    case "typ":
      return self.typ as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(key: String, typ: String, creationDate: Date? = nil, modificationDate: Date? = nil, name: String? = nil) {
    self.key = key
    self.typ = typ
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = name {
      self.name = value
    }
           
  }
       
}

class IntegrationBuilder: HCTraitBuilder {
  let type = "integration"
  let fullType = "exomind.integration"
  var error: HCBuildingError?
  let schema: HCRecordSchema = IntegrationSchema()
  var traitId: HCTraitId?

  var key:String? = nil
  var name:String?? = nil
  var modificationDate:Date?? = nil
  var creationDate:Date?? = nil
  var typ:String? = nil
  var data:[String : String]? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "data":
      return self.data as Any??
           
    case "key":
      return self.key as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "name":
      return self.name as Any??
           
    case "typ":
      return self.typ as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    case "data":
      return self.data.map { $0.mapPairs { (tup) in (tup.0, tup.1 as Any) } }
             
    default:
      return [:]
    }
  }
       
  
  init(modificationDate: Date?? = nil, key: String? = nil, name: String?? = nil, typ: String? = nil, data: [String : String]? = nil, creationDate: Date?? = nil) {
    
    if let value = name {
      self.name = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = key {
      self.key = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = typ {
      self.typ = value
    }
           
    if let value = data {
      self.data = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "data":
      
      if let arr = value as? [String : Any] {
        var casted = [ String : String ]()
        arr.forEach({ (tup) in
          if let nv = tup.1 as? String {
            casted[tup.0] = nv
          }
        })
        self.data = casted
        return
               
      } else {
        error = .invalidFieldType(name: "data", expectedType: HCMapFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "typ":
      
      if let casted = value as? String {
        self.typ = casted
        return
               
      } else {
        error = .invalidFieldType(name: "typ", expectedType: HCStringFieldType(), value: value)
      }
           
    case "key":
      
      if let casted = value as? String {
        self.key = casted
        return
               
      } else {
        error = .invalidFieldType(name: "key", expectedType: HCStringFieldType(), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "name":
      
      if let casted = value as? String {
        self.name = casted
        return
               
      } else {
        error = .invalidFieldType(name: "name", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.data == nil {
      error = .missingField(name: "data")
      return nil
    }
           
    if self.key == nil {
      error = .missingField(name: "key")
      return nil
    }
           
    if self.typ == nil {
      error = .missingField(name: "typ")
      return nil
    }
           
    let record = IntegrationFull(data: self.data!, key: self.key!, typ: self.typ!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.name {
      record.name = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.key == nil {
      error = .missingField(name: "key")
      return nil
    }
           
    if self.typ == nil {
      error = .missingField(name: "typ")
      return nil
    }
           
    let record = IntegrationSummary(key: self.key!, typ: self.typ!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.name {
      record.name = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Email: HCTrait {
  var to:[Contact] { get }
  var receivedDate:Date { get }
  var source:IntegrationSource { get }
  var subject:String? { get }
  var from:Contact { get }
  var modificationDate:Date? { get }
  var snippet:String? { get }
  var bcc:[Contact] { get }
  var id:String { get }
  var cc:[Contact] { get }
  var unread:Bool? { get }
  var creationDate:Date? { get }
}

class EmailFull: Email, HCFullRecord {
  let type = "email"
  let fullType = "exomind.email"
  let schema: HCRecordSchema = EmailSchema()
  var traitId: HCTraitId?

  var to:[Contact]
  var receivedDate:Date
  var id:String
  var snippet:String?
  var modificationDate:Date?
  var attachments:[FileAttachment]
  var unread:Bool?
  var bcc:[Contact]
  var parts:[EmailPart]
  var source:IntegrationSource
  var creationDate:Date?
  var cc:[Contact]
  var from:Contact
  var subject:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "attachments":
      return self.attachments as Any?
           
    case "bcc":
      return self.bcc as Any?
           
    case "cc":
      return self.cc as Any?
           
    case "creation_date":
      return self.creationDate as Any?
           
    case "from":
      return self.from as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "parts":
      return self.parts as Any?
           
    case "received_date":
      return self.receivedDate as Any?
           
    case "snippet":
      return self.snippet as Any?
           
    case "source":
      return self.source as Any?
           
    case "subject":
      return self.subject as Any?
           
    case "to":
      return self.to as Any?
           
    case "unread":
      return self.unread as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    case "attachments":
      return self.attachments.map { $0 as Any }
             
    case "bcc":
      return self.bcc.map { $0 as Any }
             
    case "cc":
      return self.cc.map { $0 as Any }
             
    case "parts":
      return self.parts.map { $0 as Any }
             
    case "to":
      return self.to.map { $0 as Any }
             
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(attachments: [FileAttachment], bcc: [Contact], cc: [Contact], from: Contact, id: String, parts: [EmailPart], receivedDate: Date, source: IntegrationSource, to: [Contact], creationDate: Date? = nil, modificationDate: Date? = nil, snippet: String? = nil, subject: String? = nil, unread: Bool? = nil) {
    self.attachments = attachments
    self.bcc = bcc
    self.cc = cc
    self.from = from
    self.id = id
    self.parts = parts
    self.receivedDate = receivedDate
    self.source = source
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = snippet {
      self.snippet = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
    if let value = unread {
      self.unread = value
    }
           
  }
       
}

class EmailSummary: Email, HCSummaryRecord {
  let type = "email"
  let fullType = "exomind.email"
  let schema: HCRecordSchema = EmailSchema()
  var traitId: HCTraitId?

  var to:[Contact]
  var receivedDate:Date
  var id:String
  var snippet:String?
  var modificationDate:Date?
  var unread:Bool?
  var bcc:[Contact]
  var source:IntegrationSource
  var creationDate:Date?
  var cc:[Contact]
  var from:Contact
  var subject:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "bcc":
      return self.bcc as Any?
           
    case "cc":
      return self.cc as Any?
           
    case "creation_date":
      return self.creationDate as Any?
           
    case "from":
      return self.from as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "received_date":
      return self.receivedDate as Any?
           
    case "snippet":
      return self.snippet as Any?
           
    case "source":
      return self.source as Any?
           
    case "subject":
      return self.subject as Any?
           
    case "to":
      return self.to as Any?
           
    case "unread":
      return self.unread as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    case "bcc":
      return self.bcc.map { $0 as Any }
             
    case "cc":
      return self.cc.map { $0 as Any }
             
    case "to":
      return self.to.map { $0 as Any }
             
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(bcc: [Contact], cc: [Contact], from: Contact, id: String, receivedDate: Date, source: IntegrationSource, to: [Contact], creationDate: Date? = nil, modificationDate: Date? = nil, snippet: String? = nil, subject: String? = nil, unread: Bool? = nil) {
    self.bcc = bcc
    self.cc = cc
    self.from = from
    self.id = id
    self.receivedDate = receivedDate
    self.source = source
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = snippet {
      self.snippet = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
    if let value = unread {
      self.unread = value
    }
           
  }
       
}

class EmailBuilder: HCTraitBuilder {
  let type = "email"
  let fullType = "exomind.email"
  var error: HCBuildingError?
  let schema: HCRecordSchema = EmailSchema()
  var traitId: HCTraitId?

  var from:Contact? = nil
  var attachments:[FileAttachment]? = nil
  var receivedDate:Date? = nil
  var to:[Contact]? = nil
  var source:IntegrationSource? = nil
  var id:String? = nil
  var subject:String?? = nil
  var cc:[Contact]? = nil
  var unread:Bool?? = nil
  var bcc:[Contact]? = nil
  var modificationDate:Date?? = nil
  var snippet:String?? = nil
  var creationDate:Date?? = nil
  var parts:[EmailPart]? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "attachments":
      return self.attachments as Any??
           
    case "bcc":
      return self.bcc as Any??
           
    case "cc":
      return self.cc as Any??
           
    case "creation_date":
      return self.creationDate as Any??
           
    case "from":
      return self.from as Any??
           
    case "id":
      return self.id as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "parts":
      return self.parts as Any??
           
    case "received_date":
      return self.receivedDate as Any??
           
    case "snippet":
      return self.snippet as Any??
           
    case "source":
      return self.source as Any??
           
    case "subject":
      return self.subject as Any??
           
    case "to":
      return self.to as Any??
           
    case "unread":
      return self.unread as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    case "attachments":
      return self.attachments.map { $0.map { $0 as Any } }
             
    case "bcc":
      return self.bcc.map { $0.map { $0 as Any } }
             
    case "cc":
      return self.cc.map { $0.map { $0 as Any } }
             
    case "parts":
      return self.parts.map { $0.map { $0 as Any } }
             
    case "to":
      return self.to.map { $0.map { $0 as Any } }
             
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(cc: [Contact]? = nil, from: Contact? = nil, modificationDate: Date?? = nil, parts: [EmailPart]? = nil, attachments: [FileAttachment]? = nil, bcc: [Contact]? = nil, source: IntegrationSource? = nil, snippet: String?? = nil, subject: String?? = nil, unread: Bool?? = nil, receivedDate: Date? = nil, to: [Contact]? = nil, id: String? = nil, creationDate: Date?? = nil) {
    
    if let value = bcc {
      self.bcc = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
    if let value = unread {
      self.unread = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = snippet {
      self.snippet = value
    }
           
    if let value = id {
      self.id = value
    }
           
    if let value = parts {
      self.parts = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = from {
      self.from = value
    }
           
    if let value = source {
      self.source = value
    }
           
    if let value = attachments {
      self.attachments = value
    }
           
    if let value = receivedDate {
      self.receivedDate = value
    }
           
    if let value = to {
      self.to = value
    }
           
    if let value = cc {
      self.cc = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "id":
      
      if let casted = value as? String {
        self.id = casted
        return
               
      } else {
        error = .invalidFieldType(name: "id", expectedType: HCStringFieldType(), value: value)
      }
           
    case "to":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? Contact }
        self.to = casted
        return
               
      } else {
        error = .invalidFieldType(name: "to", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")), value: value)
      }
           
    case "cc":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? Contact }
        self.cc = casted
        return
               
      } else {
        error = .invalidFieldType(name: "cc", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")), value: value)
      }
           
    case "from":
      
      if let casted = value as? Contact {
        self.from = casted
        return
               
      } else {
        error = .invalidFieldType(name: "from", expectedType: HCStructureFieldType(name: "contact"), value: value)
      }
           
    case "unread":
      
      if let casted = value as? Bool {
        self.unread = casted
        return
               
      } else {
        error = .invalidFieldType(name: "unread", expectedType: HCOptionFieldType(subtype: HCBooleanFieldType()), value: value)
      }
           
    case "parts":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? EmailPart }
        self.parts = casted
        return
               
      } else {
        error = .invalidFieldType(name: "parts", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "email_part")), value: value)
      }
           
    case "received_date":
      
      if let casted = value as? Date {
        self.receivedDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "received_date", expectedType: HCDateFieldType(), value: value)
      }
           
    case "snippet":
      
      if let casted = value as? String {
        self.snippet = casted
        return
               
      } else {
        error = .invalidFieldType(name: "snippet", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "subject":
      
      if let casted = value as? String {
        self.subject = casted
        return
               
      } else {
        error = .invalidFieldType(name: "subject", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "attachments":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? FileAttachment }
        self.attachments = casted
        return
               
      } else {
        error = .invalidFieldType(name: "attachments", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "file_attachment")), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "bcc":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? Contact }
        self.bcc = casted
        return
               
      } else {
        error = .invalidFieldType(name: "bcc", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")), value: value)
      }
           
    case "source":
      
      if let casted = value as? IntegrationSource {
        self.source = casted
        return
               
      } else {
        error = .invalidFieldType(name: "source", expectedType: HCStructureFieldType(name: "integration_source"), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.attachments == nil {
      error = .missingField(name: "attachments")
      return nil
    }
           
    if self.bcc == nil {
      error = .missingField(name: "bcc")
      return nil
    }
           
    if self.cc == nil {
      error = .missingField(name: "cc")
      return nil
    }
           
    if self.from == nil {
      error = .missingField(name: "from")
      return nil
    }
           
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.parts == nil {
      error = .missingField(name: "parts")
      return nil
    }
           
    if self.receivedDate == nil {
      error = .missingField(name: "received_date")
      return nil
    }
           
    if self.source == nil {
      error = .missingField(name: "source")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = EmailFull(attachments: self.attachments!, bcc: self.bcc!, cc: self.cc!, from: self.from!, id: self.id!, parts: self.parts!, receivedDate: self.receivedDate!, source: self.source!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.snippet {
      record.snippet = value
    }
           
    if let value = self.subject {
      record.subject = value
    }
           
    if let value = self.unread {
      record.unread = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.bcc == nil {
      error = .missingField(name: "bcc")
      return nil
    }
           
    if self.cc == nil {
      error = .missingField(name: "cc")
      return nil
    }
           
    if self.from == nil {
      error = .missingField(name: "from")
      return nil
    }
           
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.receivedDate == nil {
      error = .missingField(name: "received_date")
      return nil
    }
           
    if self.source == nil {
      error = .missingField(name: "source")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = EmailSummary(bcc: self.bcc!, cc: self.cc!, from: self.from!, id: self.id!, receivedDate: self.receivedDate!, source: self.source!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.snippet {
      record.snippet = value
    }
           
    if let value = self.subject {
      record.subject = value
    }
           
    if let value = self.unread {
      record.unread = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Link: HCTrait {
  var url:String { get }
  var modificationDate:Date? { get }
  var title:String? { get }
  var id:String { get }
  var creationDate:Date? { get }
}

class LinkFull: Link, HCFullRecord {
  let type = "link"
  let fullType = "exomind.link"
  let schema: HCRecordSchema = LinkSchema()
  var traitId: HCTraitId?

  var id:String
  var modificationDate:Date?
  var url:String
  var title:String?
  var creationDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "title":
      return self.title as Any?
           
    case "url":
      return self.url as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(id: String, url: String, creationDate: Date? = nil, modificationDate: Date? = nil, title: String? = nil) {
    self.id = id
    self.url = url
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = title {
      self.title = value
    }
           
  }
       
}

class LinkSummary: Link, HCSummaryRecord {
  let type = "link"
  let fullType = "exomind.link"
  let schema: HCRecordSchema = LinkSchema()
  var traitId: HCTraitId?

  var id:String
  var modificationDate:Date?
  var url:String
  var title:String?
  var creationDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "title":
      return self.title as Any?
           
    case "url":
      return self.url as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(id: String, url: String, creationDate: Date? = nil, modificationDate: Date? = nil, title: String? = nil) {
    self.id = id
    self.url = url
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = title {
      self.title = value
    }
           
  }
       
}

class LinkBuilder: HCTraitBuilder {
  let type = "link"
  let fullType = "exomind.link"
  var error: HCBuildingError?
  let schema: HCRecordSchema = LinkSchema()
  var traitId: HCTraitId?

  var id:String? = nil
  var title:String?? = nil
  var modificationDate:Date?? = nil
  var creationDate:Date?? = nil
  var url:String? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "id":
      return self.id as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "title":
      return self.title as Any??
           
    case "url":
      return self.url as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(modificationDate: Date?? = nil, title: String?? = nil, url: String? = nil, id: String? = nil, creationDate: Date?? = nil) {
    
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = url {
      self.url = value
    }
           
    if let value = title {
      self.title = value
    }
           
    if let value = id {
      self.id = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "id":
      
      if let casted = value as? String {
        self.id = casted
        return
               
      } else {
        error = .invalidFieldType(name: "id", expectedType: HCStringFieldType(), value: value)
      }
           
    case "url":
      
      if let casted = value as? String {
        self.url = casted
        return
               
      } else {
        error = .invalidFieldType(name: "url", expectedType: HCStringFieldType(), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "title":
      
      if let casted = value as? String {
        self.title = casted
        return
               
      } else {
        error = .invalidFieldType(name: "title", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.url == nil {
      error = .missingField(name: "url")
      return nil
    }
           
    let record = LinkFull(id: self.id!, url: self.url!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.title {
      record.title = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.url == nil {
      error = .missingField(name: "url")
      return nil
    }
           
    let record = LinkSummary(id: self.id!, url: self.url!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.title {
      record.title = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol EmailPartPlain: HCStructure {
  
}

class EmailPartPlainFull: EmailPartPlain, HCFullRecord, EmailPart {
  let type = "email_part_plain"
  let fullType = "exomind.email_part_plain"
  let schema: HCRecordSchema = EmailPartPlainSchema()
  var traitId: HCTraitId?

  var body:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "body":
      return self.body as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(body: String) {
    self.body = body
    
  }
       
}

class EmailPartPlainSummary: EmailPartPlain, HCSummaryRecord, EmailPart {
  let type = "email_part_plain"
  let fullType = "exomind.email_part_plain"
  let schema: HCRecordSchema = EmailPartPlainSchema()
  var traitId: HCTraitId?

  
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init() {
    
  }
       
}

class EmailPartPlainBuilder: HCStructureBuilder {
  let type = "email_part_plain"
  let fullType = "exomind.email_part_plain"
  var error: HCBuildingError?
  let schema: HCRecordSchema = EmailPartPlainSchema()
  var traitId: HCTraitId?

  var body:String? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "body":
      return self.body as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(body: String? = nil) {
    
    if let value = body {
      self.body = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "body":
      
      if let casted = value as? String {
        self.body = casted
        return
               
      } else {
        error = .invalidFieldType(name: "body", expectedType: HCStringFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.body == nil {
      error = .missingField(name: "body")
      return nil
    }
           
    let record = EmailPartPlainFull(body: self.body!)
    
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    let record = EmailPartPlainSummary()
    
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol DraftEmail: HCTrait {
  var sentDate:Date? { get }
  var subject:String? { get }
  var modificationDate:Date? { get }
  var sendingDate:Date? { get }
  var creationDate:Date? { get }
}

class DraftEmailFull: DraftEmail, HCFullRecord {
  let type = "draft_email"
  let fullType = "exomind.draft_email"
  let schema: HCRecordSchema = DraftEmailSchema()
  var traitId: HCTraitId?

  var to:[Contact]
  var modificationDate:Date?
  var attachments:[FileAttachment]
  var sentDate:Date?
  var bcc:[Contact]
  var parts:[EmailPart]
  var from:IntegrationSource?
  var creationDate:Date?
  var cc:[Contact]
  var inReplyTo:HCTraitId?
  var subject:String?
  var sendingDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "attachments":
      return self.attachments as Any?
           
    case "bcc":
      return self.bcc as Any?
           
    case "cc":
      return self.cc as Any?
           
    case "creation_date":
      return self.creationDate as Any?
           
    case "from":
      return self.from as Any?
           
    case "in_reply_to":
      return self.inReplyTo as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "parts":
      return self.parts as Any?
           
    case "sending_date":
      return self.sendingDate as Any?
           
    case "sent_date":
      return self.sentDate as Any?
           
    case "subject":
      return self.subject as Any?
           
    case "to":
      return self.to as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    case "attachments":
      return self.attachments.map { $0 as Any }
             
    case "bcc":
      return self.bcc.map { $0 as Any }
             
    case "cc":
      return self.cc.map { $0 as Any }
             
    case "parts":
      return self.parts.map { $0 as Any }
             
    case "to":
      return self.to.map { $0 as Any }
             
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(attachments: [FileAttachment], bcc: [Contact], cc: [Contact], parts: [EmailPart], to: [Contact], creationDate: Date? = nil, from: IntegrationSource? = nil, inReplyTo: HCTraitId? = nil, modificationDate: Date? = nil, sendingDate: Date? = nil, sentDate: Date? = nil, subject: String? = nil) {
    self.attachments = attachments
    self.bcc = bcc
    self.cc = cc
    self.parts = parts
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = from {
      self.from = value
    }
           
    if let value = inReplyTo {
      self.inReplyTo = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = sendingDate {
      self.sendingDate = value
    }
           
    if let value = sentDate {
      self.sentDate = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
  }
       
}

class DraftEmailSummary: DraftEmail, HCSummaryRecord {
  let type = "draft_email"
  let fullType = "exomind.draft_email"
  let schema: HCRecordSchema = DraftEmailSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var sentDate:Date?
  var creationDate:Date?
  var subject:String?
  var sendingDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "sending_date":
      return self.sendingDate as Any?
           
    case "sent_date":
      return self.sentDate as Any?
           
    case "subject":
      return self.subject as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(creationDate: Date? = nil, modificationDate: Date? = nil, sendingDate: Date? = nil, sentDate: Date? = nil, subject: String? = nil) {
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = sendingDate {
      self.sendingDate = value
    }
           
    if let value = sentDate {
      self.sentDate = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
  }
       
}

class DraftEmailBuilder: HCTraitBuilder {
  let type = "draft_email"
  let fullType = "exomind.draft_email"
  var error: HCBuildingError?
  let schema: HCRecordSchema = DraftEmailSchema()
  var traitId: HCTraitId?

  var attachments:[FileAttachment]? = nil
  var to:[Contact]? = nil
  var subject:String?? = nil
  var cc:[Contact]? = nil
  var bcc:[Contact]? = nil
  var modificationDate:Date?? = nil
  var creationDate:Date?? = nil
  var inReplyTo:HCTraitId?? = nil
  var sentDate:Date?? = nil
  var sendingDate:Date?? = nil
  var from:IntegrationSource?? = nil
  var parts:[EmailPart]? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "attachments":
      return self.attachments as Any??
           
    case "bcc":
      return self.bcc as Any??
           
    case "cc":
      return self.cc as Any??
           
    case "creation_date":
      return self.creationDate as Any??
           
    case "from":
      return self.from as Any??
           
    case "in_reply_to":
      return self.inReplyTo as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "parts":
      return self.parts as Any??
           
    case "sending_date":
      return self.sendingDate as Any??
           
    case "sent_date":
      return self.sentDate as Any??
           
    case "subject":
      return self.subject as Any??
           
    case "to":
      return self.to as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    case "attachments":
      return self.attachments.map { $0.map { $0 as Any } }
             
    case "bcc":
      return self.bcc.map { $0.map { $0 as Any } }
             
    case "cc":
      return self.cc.map { $0.map { $0 as Any } }
             
    case "parts":
      return self.parts.map { $0.map { $0 as Any } }
             
    case "to":
      return self.to.map { $0.map { $0 as Any } }
             
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(cc: [Contact]? = nil, modificationDate: Date?? = nil, parts: [EmailPart]? = nil, attachments: [FileAttachment]? = nil, bcc: [Contact]? = nil, subject: String?? = nil, sentDate: Date?? = nil, inReplyTo: HCTraitId?? = nil, sendingDate: Date?? = nil, from: IntegrationSource?? = nil, to: [Contact]? = nil, creationDate: Date?? = nil) {
    
    if let value = bcc {
      self.bcc = value
    }
           
    if let value = subject {
      self.subject = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = inReplyTo {
      self.inReplyTo = value
    }
           
    if let value = parts {
      self.parts = value
    }
           
    if let value = sendingDate {
      self.sendingDate = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = from {
      self.from = value
    }
           
    if let value = attachments {
      self.attachments = value
    }
           
    if let value = to {
      self.to = value
    }
           
    if let value = cc {
      self.cc = value
    }
           
    if let value = sentDate {
      self.sentDate = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "to":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? Contact }
        self.to = casted
        return
               
      } else {
        error = .invalidFieldType(name: "to", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")), value: value)
      }
           
    case "in_reply_to":
      
      if let casted = value as? HCTraitId {
        self.inReplyTo = casted
        return
               
      } else {
        error = .invalidFieldType(name: "in_reply_to", expectedType: HCOptionFieldType(subtype: HCTraitReferenceFieldType()), value: value)
      }
           
    case "cc":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? Contact }
        self.cc = casted
        return
               
      } else {
        error = .invalidFieldType(name: "cc", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")), value: value)
      }
           
    case "sent_date":
      
      if let casted = value as? Date {
        self.sentDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "sent_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "parts":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? EmailPart }
        self.parts = casted
        return
               
      } else {
        error = .invalidFieldType(name: "parts", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "email_part")), value: value)
      }
           
    case "sending_date":
      
      if let casted = value as? Date {
        self.sendingDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "sending_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "subject":
      
      if let casted = value as? String {
        self.subject = casted
        return
               
      } else {
        error = .invalidFieldType(name: "subject", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "attachments":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? FileAttachment }
        self.attachments = casted
        return
               
      } else {
        error = .invalidFieldType(name: "attachments", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "file_attachment")), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "bcc":
      
      if let arr = value as? [Any] {
        let casted = arr.compactMap { $0 as? Contact }
        self.bcc = casted
        return
               
      } else {
        error = .invalidFieldType(name: "bcc", expectedType: HCArrayFieldType(subtype: HCStructureFieldType(name: "contact")), value: value)
      }
           
    case "from":
      
      if let casted = value as? IntegrationSource {
        self.from = casted
        return
               
      } else {
        error = .invalidFieldType(name: "from", expectedType: HCOptionFieldType(subtype: HCStructureFieldType(name: "integration_source")), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.attachments == nil {
      error = .missingField(name: "attachments")
      return nil
    }
           
    if self.bcc == nil {
      error = .missingField(name: "bcc")
      return nil
    }
           
    if self.cc == nil {
      error = .missingField(name: "cc")
      return nil
    }
           
    if self.parts == nil {
      error = .missingField(name: "parts")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = DraftEmailFull(attachments: self.attachments!, bcc: self.bcc!, cc: self.cc!, parts: self.parts!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.from {
      record.from = value
    }
           
    if let value = self.inReplyTo {
      record.inReplyTo = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.sendingDate {
      record.sendingDate = value
    }
           
    if let value = self.sentDate {
      record.sentDate = value
    }
           
    if let value = self.subject {
      record.subject = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    let record = DraftEmailSummary()
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.sendingDate {
      record.sendingDate = value
    }
           
    if let value = self.sentDate {
      record.sentDate = value
    }
           
    if let value = self.subject {
      record.subject = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol OldChild: HCTrait {
  var creationDate:Date? { get }
  var modificationDate:Date? { get }
  var to:HCEntityId { get }
  var date:Date { get }
}

class OldChildFull: OldChild, HCFullRecord {
  let type = "old_child"
  let fullType = "exomind.old_child"
  let schema: HCRecordSchema = OldChildSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var to:HCEntityId
  var date:Date
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "date":
      return self.date as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "to":
      return self.to as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(date: Date, to: HCEntityId, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.date = date
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class OldChildSummary: OldChild, HCSummaryRecord {
  let type = "old_child"
  let fullType = "exomind.old_child"
  let schema: HCRecordSchema = OldChildSchema()
  var traitId: HCTraitId?

  var creationDate:Date?
  var modificationDate:Date?
  var to:HCEntityId
  var date:Date
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "date":
      return self.date as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "to":
      return self.to as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(date: Date, to: HCEntityId, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.date = date
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class OldChildBuilder: HCTraitBuilder {
  let type = "old_child"
  let fullType = "exomind.old_child"
  var error: HCBuildingError?
  let schema: HCRecordSchema = OldChildSchema()
  var traitId: HCTraitId?

  var creationDate:Date?? = nil
  var modificationDate:Date?? = nil
  var to:HCEntityId? = nil
  var date:Date? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "date":
      return self.date as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "to":
      return self.to as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(creationDate: Date?? = nil, modificationDate: Date?? = nil, to: HCEntityId? = nil, date: Date? = nil) {
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = to {
      self.to = value
    }
           
    if let value = date {
      self.date = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "to":
      
      if let casted = value as? HCEntityId {
        self.to = casted
        return
               
      } else {
        error = .invalidFieldType(name: "to", expectedType: HCEntityReferenceFieldType(), value: value)
      }
           
    case "date":
      
      if let casted = value as? Date {
        self.date = casted
        return
               
      } else {
        error = .invalidFieldType(name: "date", expectedType: HCDateFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.date == nil {
      error = .missingField(name: "date")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = OldChildFull(date: self.date!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.date == nil {
      error = .missingField(name: "date")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = OldChildSummary(date: self.date!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol FileAttachmentIntegration: HCStructure {
  var integrationKey:String { get }
  var key:String { get }
  var name:String? { get }
  var mime:String? { get }
  var integrationName:String { get }
}

class FileAttachmentIntegrationFull: FileAttachmentIntegration, HCFullRecord, FileAttachment {
  let type = "file_attachment_integration"
  let fullType = "exomind.file_attachment_integration"
  let schema: HCRecordSchema = FileAttachmentIntegrationSchema()
  var traitId: HCTraitId?

  var integrationKey:String
  var inlinePlaceholder:String?
  var mime:String?
  var integrationName:String
  var name:String?
  var size:Int64?
  var data:[String : String]
  var key:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "data":
      return self.data as Any?
           
    case "inline_placeholder":
      return self.inlinePlaceholder as Any?
           
    case "integration_key":
      return self.integrationKey as Any?
           
    case "integration_name":
      return self.integrationName as Any?
           
    case "key":
      return self.key as Any?
           
    case "mime":
      return self.mime as Any?
           
    case "name":
      return self.name as Any?
           
    case "size":
      return self.size as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    case "data":
      return self.data.mapPairs { (tup) in (tup.0, tup.1 as Any) }
             
    default:
      return [:]
    }
  }
       
  
  init(data: [String : String], integrationKey: String, integrationName: String, key: String, inlinePlaceholder: String? = nil, mime: String? = nil, name: String? = nil, size: Int64? = nil) {
    self.data = data
    self.integrationKey = integrationKey
    self.integrationName = integrationName
    self.key = key
    
    if let value = inlinePlaceholder {
      self.inlinePlaceholder = value
    }
           
    if let value = mime {
      self.mime = value
    }
           
    if let value = name {
      self.name = value
    }
           
    if let value = size {
      self.size = value
    }
           
  }
       
}

class FileAttachmentIntegrationSummary: FileAttachmentIntegration, HCSummaryRecord, FileAttachment {
  let type = "file_attachment_integration"
  let fullType = "exomind.file_attachment_integration"
  let schema: HCRecordSchema = FileAttachmentIntegrationSchema()
  var traitId: HCTraitId?

  var integrationKey:String
  var mime:String?
  var integrationName:String
  var name:String?
  var key:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "integration_key":
      return self.integrationKey as Any?
           
    case "integration_name":
      return self.integrationName as Any?
           
    case "key":
      return self.key as Any?
           
    case "mime":
      return self.mime as Any?
           
    case "name":
      return self.name as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(integrationKey: String, integrationName: String, key: String, mime: String? = nil, name: String? = nil) {
    self.integrationKey = integrationKey
    self.integrationName = integrationName
    self.key = key
    
    if let value = mime {
      self.mime = value
    }
           
    if let value = name {
      self.name = value
    }
           
  }
       
}

class FileAttachmentIntegrationBuilder: HCStructureBuilder {
  let type = "file_attachment_integration"
  let fullType = "exomind.file_attachment_integration"
  var error: HCBuildingError?
  let schema: HCRecordSchema = FileAttachmentIntegrationSchema()
  var traitId: HCTraitId?

  var key:String? = nil
  var mime:String?? = nil
  var name:String?? = nil
  var integrationKey:String? = nil
  var size:Int64?? = nil
  var integrationName:String? = nil
  var inlinePlaceholder:String?? = nil
  var data:[String : String]? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "data":
      return self.data as Any??
           
    case "inline_placeholder":
      return self.inlinePlaceholder as Any??
           
    case "integration_key":
      return self.integrationKey as Any??
           
    case "integration_name":
      return self.integrationName as Any??
           
    case "key":
      return self.key as Any??
           
    case "mime":
      return self.mime as Any??
           
    case "name":
      return self.name as Any??
           
    case "size":
      return self.size as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    case "data":
      return self.data.map { $0.mapPairs { (tup) in (tup.0, tup.1 as Any) } }
             
    default:
      return [:]
    }
  }
       
  
  init(mime: String?? = nil, key: String? = nil, name: String?? = nil, data: [String : String]? = nil, size: Int64?? = nil, integrationKey: String? = nil, inlinePlaceholder: String?? = nil, integrationName: String? = nil) {
    
    if let value = name {
      self.name = value
    }
           
    if let value = size {
      self.size = value
    }
           
    if let value = key {
      self.key = value
    }
           
    if let value = mime {
      self.mime = value
    }
           
    if let value = integrationKey {
      self.integrationKey = value
    }
           
    if let value = inlinePlaceholder {
      self.inlinePlaceholder = value
    }
           
    if let value = data {
      self.data = value
    }
           
    if let value = integrationName {
      self.integrationName = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "mime":
      
      if let casted = value as? String {
        self.mime = casted
        return
               
      } else {
        error = .invalidFieldType(name: "mime", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "integration_key":
      
      if let casted = value as? String {
        self.integrationKey = casted
        return
               
      } else {
        error = .invalidFieldType(name: "integration_key", expectedType: HCStringFieldType(), value: value)
      }
           
    case "integration_name":
      
      if let casted = value as? String {
        self.integrationName = casted
        return
               
      } else {
        error = .invalidFieldType(name: "integration_name", expectedType: HCStringFieldType(), value: value)
      }
           
    case "data":
      
      if let arr = value as? [String : Any] {
        var casted = [ String : String ]()
        arr.forEach({ (tup) in
          if let nv = tup.1 as? String {
            casted[tup.0] = nv
          }
        })
        self.data = casted
        return
               
      } else {
        error = .invalidFieldType(name: "data", expectedType: HCMapFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "key":
      
      if let casted = value as? String {
        self.key = casted
        return
               
      } else {
        error = .invalidFieldType(name: "key", expectedType: HCStringFieldType(), value: value)
      }
           
    case "inline_placeholder":
      
      if let casted = value as? String {
        self.inlinePlaceholder = casted
        return
               
      } else {
        error = .invalidFieldType(name: "inline_placeholder", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "name":
      
      if let casted = value as? String {
        self.name = casted
        return
               
      } else {
        error = .invalidFieldType(name: "name", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "size":
      
      if let casted = value as? Int64 {
        self.size = casted
        return
               
      } else {
        error = .invalidFieldType(name: "size", expectedType: HCOptionFieldType(subtype: HCLongFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.data == nil {
      error = .missingField(name: "data")
      return nil
    }
           
    if self.integrationKey == nil {
      error = .missingField(name: "integration_key")
      return nil
    }
           
    if self.integrationName == nil {
      error = .missingField(name: "integration_name")
      return nil
    }
           
    if self.key == nil {
      error = .missingField(name: "key")
      return nil
    }
           
    let record = FileAttachmentIntegrationFull(data: self.data!, integrationKey: self.integrationKey!, integrationName: self.integrationName!, key: self.key!)
    
    if let value = self.inlinePlaceholder {
      record.inlinePlaceholder = value
    }
           
    if let value = self.mime {
      record.mime = value
    }
           
    if let value = self.name {
      record.name = value
    }
           
    if let value = self.size {
      record.size = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.integrationKey == nil {
      error = .missingField(name: "integration_key")
      return nil
    }
           
    if self.integrationName == nil {
      error = .missingField(name: "integration_name")
      return nil
    }
           
    if self.key == nil {
      error = .missingField(name: "key")
      return nil
    }
           
    let record = FileAttachmentIntegrationSummary(integrationKey: self.integrationKey!, integrationName: self.integrationName!, key: self.key!)
    
    if let value = self.mime {
      record.mime = value
    }
           
    if let value = self.name {
      record.name = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Note: HCTrait {
  var title:String { get }
  var creationDate:Date? { get }
  var modificationDate:Date? { get }
  var id:String { get }
}

class NoteFull: Note, HCFullRecord {
  let type = "note"
  let fullType = "exomind.note"
  let schema: HCRecordSchema = NoteSchema()
  var traitId: HCTraitId?

  var title:String
  var id:String
  var modificationDate:Date?
  var creationDate:Date?
  var content:String?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "content":
      return self.content as Any?
           
    case "creation_date":
      return self.creationDate as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "title":
      return self.title as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(id: String, title: String, content: String? = nil, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.id = id
    self.title = title
    
    if let value = content {
      self.content = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class NoteSummary: Note, HCSummaryRecord {
  let type = "note"
  let fullType = "exomind.note"
  let schema: HCRecordSchema = NoteSchema()
  var traitId: HCTraitId?

  var title:String
  var creationDate:Date?
  var modificationDate:Date?
  var id:String
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "id":
      return self.id as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "title":
      return self.title as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(id: String, title: String, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.id = id
    self.title = title
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class NoteBuilder: HCTraitBuilder {
  let type = "note"
  let fullType = "exomind.note"
  var error: HCBuildingError?
  let schema: HCRecordSchema = NoteSchema()
  var traitId: HCTraitId?

  var title:String? = nil
  var id:String? = nil
  var modificationDate:Date?? = nil
  var creationDate:Date?? = nil
  var content:String?? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "content":
      return self.content as Any??
           
    case "creation_date":
      return self.creationDate as Any??
           
    case "id":
      return self.id as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "title":
      return self.title as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(modificationDate: Date?? = nil, content: String?? = nil, title: String? = nil, id: String? = nil, creationDate: Date?? = nil) {
    
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = content {
      self.content = value
    }
           
    if let value = title {
      self.title = value
    }
           
    if let value = id {
      self.id = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "content":
      
      if let casted = value as? String {
        self.content = casted
        return
               
      } else {
        error = .invalidFieldType(name: "content", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "id":
      
      if let casted = value as? String {
        self.id = casted
        return
               
      } else {
        error = .invalidFieldType(name: "id", expectedType: HCStringFieldType(), value: value)
      }
           
    case "title":
      
      if let casted = value as? String {
        self.title = casted
        return
               
      } else {
        error = .invalidFieldType(name: "title", expectedType: HCStringFieldType(), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.title == nil {
      error = .missingField(name: "title")
      return nil
    }
           
    let record = NoteFull(id: self.id!, title: self.title!)
    
    if let value = self.content {
      record.content = value
    }
           
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.id == nil {
      error = .missingField(name: "id")
      return nil
    }
           
    if self.title == nil {
      error = .missingField(name: "title")
      return nil
    }
           
    let record = NoteSummary(id: self.id!, title: self.title!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Lineage: HCTrait {
  var to:HCEntityId { get }
  var depth:Int64 { get }
  var processedDate:Date { get }
  var modificationDate:Date? { get }
  var parentName:String? { get }
  var creationDate:Date? { get }
}

class LineageFull: Lineage, HCFullRecord {
  let type = "lineage"
  let fullType = "exomind.lineage"
  let schema: HCRecordSchema = LineageSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var processedDate:Date
  var to:HCEntityId
  var depth:Int64
  var parentName:String?
  var creationDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "depth":
      return self.depth as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "parent_name":
      return self.parentName as Any?
           
    case "processed_date":
      return self.processedDate as Any?
           
    case "to":
      return self.to as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(depth: Int64, processedDate: Date, to: HCEntityId, creationDate: Date? = nil, modificationDate: Date? = nil, parentName: String? = nil) {
    self.depth = depth
    self.processedDate = processedDate
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = parentName {
      self.parentName = value
    }
           
  }
       
}

class LineageSummary: Lineage, HCSummaryRecord {
  let type = "lineage"
  let fullType = "exomind.lineage"
  let schema: HCRecordSchema = LineageSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var processedDate:Date
  var to:HCEntityId
  var depth:Int64
  var parentName:String?
  var creationDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "depth":
      return self.depth as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "parent_name":
      return self.parentName as Any?
           
    case "processed_date":
      return self.processedDate as Any?
           
    case "to":
      return self.to as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(depth: Int64, processedDate: Date, to: HCEntityId, creationDate: Date? = nil, modificationDate: Date? = nil, parentName: String? = nil) {
    self.depth = depth
    self.processedDate = processedDate
    self.to = to
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = parentName {
      self.parentName = value
    }
           
  }
       
}

class LineageBuilder: HCTraitBuilder {
  let type = "lineage"
  let fullType = "exomind.lineage"
  var error: HCBuildingError?
  let schema: HCRecordSchema = LineageSchema()
  var traitId: HCTraitId?

  var to:HCEntityId? = nil
  var depth:Int64? = nil
  var parentName:String?? = nil
  var modificationDate:Date?? = nil
  var creationDate:Date?? = nil
  var processedDate:Date? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "depth":
      return self.depth as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "parent_name":
      return self.parentName as Any??
           
    case "processed_date":
      return self.processedDate as Any??
           
    case "to":
      return self.to as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(depth: Int64? = nil, modificationDate: Date?? = nil, parentName: String?? = nil, to: HCEntityId? = nil, processedDate: Date? = nil, creationDate: Date?? = nil) {
    
    if let value = processedDate {
      self.processedDate = value
    }
           
    if let value = parentName {
      self.parentName = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = depth {
      self.depth = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = to {
      self.to = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "processed_date":
      
      if let casted = value as? Date {
        self.processedDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "processed_date", expectedType: HCDateFieldType(), value: value)
      }
           
    case "parent_name":
      
      if let casted = value as? String {
        self.parentName = casted
        return
               
      } else {
        error = .invalidFieldType(name: "parent_name", expectedType: HCOptionFieldType(subtype: HCStringFieldType()), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "depth":
      
      if let casted = value as? Int64 {
        self.depth = casted
        return
               
      } else {
        error = .invalidFieldType(name: "depth", expectedType: HCLongFieldType(), value: value)
      }
           
    case "to":
      
      if let casted = value as? HCEntityId {
        self.to = casted
        return
               
      } else {
        error = .invalidFieldType(name: "to", expectedType: HCEntityReferenceFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.depth == nil {
      error = .missingField(name: "depth")
      return nil
    }
           
    if self.processedDate == nil {
      error = .missingField(name: "processed_date")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = LineageFull(depth: self.depth!, processedDate: self.processedDate!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.parentName {
      record.parentName = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.depth == nil {
      error = .missingField(name: "depth")
      return nil
    }
           
    if self.processedDate == nil {
      error = .missingField(name: "processed_date")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    let record = LineageSummary(depth: self.depth!, processedDate: self.processedDate!, to: self.to!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
    if let value = self.parentName {
      record.parentName = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     

protocol Child: HCTrait {
  var date:Date { get }
  var to:HCEntityId { get }
  var weight:Int64 { get }
  var modificationDate:Date? { get }
  var creationDate:Date? { get }
}

class ChildFull: Child, HCFullRecord {
  let type = "child"
  let fullType = "exomind.child"
  let schema: HCRecordSchema = ChildSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var date:Date
  var to:HCEntityId
  var weight:Int64
  var creationDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "date":
      return self.date as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "to":
      return self.to as Any?
           
    case "weight":
      return self.weight as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(date: Date, to: HCEntityId, weight: Int64, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.date = date
    self.to = to
    self.weight = weight
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class ChildSummary: Child, HCSummaryRecord {
  let type = "child"
  let fullType = "exomind.child"
  let schema: HCRecordSchema = ChildSchema()
  var traitId: HCTraitId?

  var modificationDate:Date?
  var date:Date
  var to:HCEntityId
  var weight:Int64
  var creationDate:Date?
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any?
           
    case "date":
      return self.date as Any?
           
    case "modification_date":
      return self.modificationDate as Any?
           
    case "to":
      return self.to as Any?
           
    case "weight":
      return self.weight as Any?
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(date: Date, to: HCEntityId, weight: Int64, creationDate: Date? = nil, modificationDate: Date? = nil) {
    self.date = date
    self.to = to
    self.weight = weight
    
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
  }
       
}

class ChildBuilder: HCTraitBuilder {
  let type = "child"
  let fullType = "exomind.child"
  var error: HCBuildingError?
  let schema: HCRecordSchema = ChildSchema()
  var traitId: HCTraitId?

  var to:HCEntityId? = nil
  var weight:Int64? = nil
  var modificationDate:Date?? = nil
  var date:Date? = nil
  var creationDate:Date?? = nil
  
  func get(_ name: String) -> Any?? {
    switch name {
    
    case "creation_date":
      return self.creationDate as Any??
           
    case "date":
      return self.date as Any??
           
    case "modification_date":
      return self.modificationDate as Any??
           
    case "to":
      return self.to as Any??
           
    case "weight":
      return self.weight as Any??
           
    default:
      return nil
    }
  }
  func getArray(_ name: String) -> [Any]? {
    switch name {
    
    default:
      return []
    }
  }
  func getMap(_ name: String) -> [String:Any]? {
    switch name {
    
    default:
      return [:]
    }
  }
       
  
  init(weight: Int64? = nil, modificationDate: Date?? = nil, to: HCEntityId? = nil, date: Date? = nil, creationDate: Date?? = nil) {
    
    if let value = date {
      self.date = value
    }
           
    if let value = modificationDate {
      self.modificationDate = value
    }
           
    if let value = creationDate {
      self.creationDate = value
    }
           
    if let value = weight {
      self.weight = value
    }
           
    if let value = to {
      self.to = value
    }
           
  }
       
  
  func set(_ name: String, value: Any) {
    switch name {
    
    case "modification_date":
      
      if let casted = value as? Date {
        self.modificationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "modification_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "date":
      
      if let casted = value as? Date {
        self.date = casted
        return
               
      } else {
        error = .invalidFieldType(name: "date", expectedType: HCDateFieldType(), value: value)
      }
           
    case "weight":
      
      if let casted = value as? Int64 {
        self.weight = casted
        return
               
      } else {
        error = .invalidFieldType(name: "weight", expectedType: HCLongFieldType(), value: value)
      }
           
    case "creation_date":
      
      if let casted = value as? Date {
        self.creationDate = casted
        return
               
      } else {
        error = .invalidFieldType(name: "creation_date", expectedType: HCOptionFieldType(subtype: HCDateFieldType()), value: value)
      }
           
    case "to":
      
      if let casted = value as? HCEntityId {
        self.to = casted
        return
               
      } else {
        error = .invalidFieldType(name: "to", expectedType: HCEntityReferenceFieldType(), value: value)
      }
           
    default:
      print("Field \(name) not found")
    }
  }
       

  func build() -> HCFullRecord? {
    
    
    if self.date == nil {
      error = .missingField(name: "date")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    if self.weight == nil {
      error = .missingField(name: "weight")
      return nil
    }
           
    let record = ChildFull(date: self.date!, to: self.to!, weight: self.weight!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
  func buildSummary() -> HCSummaryRecord? {
    
    
    if self.date == nil {
      error = .missingField(name: "date")
      return nil
    }
           
    if self.to == nil {
      error = .missingField(name: "to")
      return nil
    }
           
    if self.weight == nil {
      error = .missingField(name: "weight")
      return nil
    }
           
    let record = ChildSummary(date: self.date!, to: self.to!, weight: self.weight!)
    
    if let value = self.creationDate {
      record.creationDate = value
    }
           
    if let value = self.modificationDate {
      record.modificationDate = value
    }
           
         
    record.traitId = self.traitId
    return record
  }
}
     
     
