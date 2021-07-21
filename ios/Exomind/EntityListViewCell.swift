import SwiftUI

class EntityListViewCellHost: UITableViewCell {
    var host: UIHostingController<AnyView>?
}

struct EntityListViewCell: View {
    let data: EntityListCellData 

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                ZStack {
                    Color(self.data.color)
                            .frame(width: 40, height: 40, alignment: .center)
                            .cornerRadius(24)

                    Image(uiImage: self.data.image)
                }.frame(width: 40).padding(5) // sync pills

                VStack(alignment: .leading, spacing: 5) {
                    HStack {
                        Text(self.data.line1)
                                .font(.system(size: 14))
                                .lineLimit(1)

                        Spacer()

                        Text(self.data.date)
                                .font(.system(size: 12))
                    }

                    Text(self.data.line2)
                        .font(.system(size: 14))
                        .lineLimit(1)

                    if self.data.line3 != "" {
                        Text(self.data.line3)
                                .lineLimit(1)
                                .font(.system(size: 12))
                    }
                }.padding(3).frame(alignment: .topTrailing)
            }

            if !self.data.collections.isEmpty {
                HStack(spacing: 0) {
                    Spacer().frame(width: 40).padding(5) // sync with image

                    CollectionPillsView(collections: self.data.collections)
                }
            }
        }.padding(.vertical, 5)
    }

}

struct EntityListCellData {
    let image: UIImage
    let date: String
    let color: UIColor

    var line1: String = ""
    var line2: String = ""
    var line3: String = ""

    var collections: [CollectionPillData] = []

    init(image: UIImage, date: Date, color: UIColor, title: String, collections: [CollectionPillData] = []) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = " "
        self.line2 = title
        self.line3 = " "
        self.collections = collections
    }

    init(image: UIImage, date: Date, color: UIColor, title: String, subtitle: String, collections: [CollectionPillData] = []) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = title
        self.line2 = subtitle
        self.line3 = " "
        self.collections = collections
    }

    init(image: UIImage, date: Date, color: UIColor, title: String, subtitle: String, text: String, collections: [CollectionPillData] = []) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = title
        self.line2 = subtitle
        self.line3 = text
        self.collections = collections
    }
}

struct EntityListViewCell_Previews: PreviewProvider {
    static var previews: some View {
        let img = ObjectsIcon.icon(forFontAwesome: .addressBook, color: .white, dimension: 24)
        let date = Date()

        List {
            EntityListViewCell(data: EntityListCellData(image: img, date: date, color: UIColor.red, title: "Title"))

            EntityListViewCell(data: EntityListCellData(image: img, date: date, color: UIColor.green, title: "Title", subtitle: "Sub title"))

            EntityListViewCell(data: EntityListCellData(image: img, date: date, color: UIColor.orange, title: "Title", subtitle: "Sub title", text: "long long long long long long long long long long text"))

            EntityListViewCell(data: EntityListCellData(image: img, date: date, color: UIColor.systemPink, title: "Title", subtitle: "Subtitle", text: "Some text", collections: [CollectionPillData(id: "col1", name: "Collection 1")]))

            EntityListViewCell(data: EntityListCellData(image: img, date: date, color: UIColor.systemGray, title: "Title", collections: [CollectionPillData(id: "col1", name: "Collection 1")]))
        }
    }
}
