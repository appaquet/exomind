

import SwiftUI

struct CollectionSelectorCell: View {
    static let ICON_SIZE = CGFloat(20)

    let data: CollectionSelectorCellData

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 0) {
                Image(uiImage: self.data.finalIcon())
                    .padding(10)

                Text(self.data.name)
                    .lineLimit(1)

                Spacer()

                if self.data.checked {
                    Text("✓")
                        .font(.system(size: 20))
                        .padding(.trailing, 10)
                }
            }

            if !self.data.parents.isEmpty {
                CollectionPillsView(collections: self.data.parents)
                    .padding(.leading, 40)
                    .padding(.bottom, 5)
            }
        }
    }
}

struct CollectionSelectorCellData {
    let id: String
    let name: String
    var checked: Bool = false
    var icon: UIImage? = nil
    var parents: [CollectionPillData] = []

    func finalIcon() -> UIImage {
        if let icon = self.icon {
            return icon
        } else {
            return ObjectsIcon.icon(forFontAwesome: .folderOpen, color: .black, dimension: CollectionSelectorCell.ICON_SIZE)
        }
    }
}


struct CollectionSelectorCell_Previews: PreviewProvider {
    static var previews: some View {
        let img = ObjectsIcon.icon(forFontAwesome: .addressBook, color: .black, dimension: CollectionSelectorCell.ICON_SIZE)

        List {
            CollectionSelectorCell(data: CollectionSelectorCellData(id: "id1", name: "Hello", checked: true))

            CollectionSelectorCell(data: CollectionSelectorCellData(id: "id2", name: "Hello world how are you doing today and long long long", checked: true))

            CollectionSelectorCell(data: CollectionSelectorCellData(id: "id3", name: "Hello", icon: img))

            CollectionSelectorCell(data: CollectionSelectorCellData(id: "id4", name: "Hello", parents: [
                CollectionPillData(id: "parent1", name: "parent1"),
                CollectionPillData(id: "parent2", name: "parent2")
            ]))

            CollectionSelectorCell(data: CollectionSelectorCellData(id: "id5", name: "Hello", checked: true, parents: [
                CollectionPillData(id: "parent1", name: "parent1"),
                CollectionPillData(id: "parent2", name: "parent2")
            ]))
        }
    }
}
