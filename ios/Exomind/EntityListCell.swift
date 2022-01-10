import SwiftUI

struct EntityListCell: View {
    let data: EntityListCellData

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                ZStack {
                    Color(self.data.color)
                            .frame(width: 40, height: 40, alignment: .center)
                            .cornerRadius(24)

                    Image(uiImage: self.data.image)
                }.frame(width: 40).padding(5) // sync with pills

                VStack(alignment: .leading, spacing: 5) {
                    HStack {
                        Text(self.data.line1)
                            .font(.system(size: 14, weight: self.data.weight))
                            .lineLimit(1)

                        Spacer()

                        Text(self.data.date)
                                .font(.system(size: 12))
                    }

                    HStack(spacing: 0) {
                        Text(self.data.line2)
                                .font(.system(size: 14, weight: self.data.weight))
                                .lineLimit(1)
                        
                        if !self.data.indicators.isEmpty {
                            Spacer()
                            
                            ForEach(self.data.indicators, id: \.self) { icon in
                                Image(uiImage: icon)
                            }
                        }
                    }

                    if self.data.line3 != "" {
                        Text(self.data.line3)
                            .lineLimit(1)
                            .font(.system(size: 11, weight: self.data.weight))
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

class EntityListCellData {
    let image: UIImage
    let date: String
    let color: UIColor

    var line1: String = ""
    var line2: String = ""
    var line3: String = ""
    var weight: Font.Weight = .regular

    var collections: [CollectionPillData] = []
    var indicators: [UIImage] = []

    init(image: UIImage, date: Date, color: UIColor, title: String) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = " "
        self.line2 = title
        self.line3 = " "
    }

    init(image: UIImage, date: Date, color: UIColor, title: String, subtitle: String) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = title
        self.line2 = subtitle
        self.line3 = " "
    }

    init(image: UIImage, date: Date, color: UIColor, title: String, text: String) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = " "
        self.line2 = title
        self.line3 = text
    }

    init(image: UIImage, date: Date, color: UIColor, title: String, subtitle: String, text: String) {
        self.image = image
        self.date = date.toShort()
        self.color = color
        self.line1 = title
        self.line2 = subtitle
        self.line3 = text
    }

    func withCollections(_ collections: [CollectionPillData]) -> Self {
        self.collections = collections
        return self
    }

    func withBold(enabled: Bool = true) -> Self {
        if enabled {
            self.weight = .bold
        } else {
            self.weight = .regular
        }
        return self
    }

    func withIndicators(_ indicators: [UIImage]) -> Self {
        self.indicators = indicators
        return self
    }
}

struct EntityListViewCell_Previews: PreviewProvider {
    static var previews: some View {
        let img = ObjectsIcon.icon(forFontAwesome: .addressBook, color: .white, dimension: 24)
        let ind1 = ObjectsIcon.icon(forFontAwesome: .clock, color: .lightGray, dimension: 16)
        let ind2 = ObjectsIcon.icon(forFontAwesome: .thumbtack, color: .lightGray, dimension: 16)
        let date = Date()

        List {
            EntityListCell(data: EntityListCellData(image: img, date: date, color: UIColor.red, title: "Title"))

            EntityListCell(data: EntityListCellData(image: img, date: date, color: UIColor.green, title: "Title", subtitle: "Sub title"))

            EntityListCell(data: EntityListCellData(image: img, date: date, color: UIColor.orange, title: "Title", subtitle: "Sub title", text: "long long long long long long long long long long text"))

            EntityListCell(data: EntityListCellData(image: img, date: date, color: UIColor.systemPink, title: "Title", subtitle: "Subtitle", text: "Some text").withCollections([CollectionPillData(id: "col1", name: "Collection 1")]))

            EntityListCell(data: EntityListCellData(image: img, date: date, color: UIColor.systemGray, title: "Title").withCollections([CollectionPillData(id: "col1", name: "Collection 1")]).withBold())

            EntityListCell(data: EntityListCellData(image: img, date: date, color: UIColor.green, title: "Title", subtitle: "Sub title with long long long long text so that it gets to the indicators").withIndicators([ind1, ind2]))
        }
    }
}
