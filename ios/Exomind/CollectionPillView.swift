import SwiftUI
import Introspect

struct CollectionPillView: View {
    static let ICON_SIZE = CGFloat(12)

    private let collection: CollectionPillData

    init(collection: CollectionPillData) {
        self.collection = collection
    }

    var body: some View {
        HStack(spacing: 2) {
            self.iconsView()

            Text(self.collection.name)
                .font(.system(size: 12))
                .frame(maxWidth: 120, alignment: .leading)
                .fixedSize(horizontal: true, vertical: true) // text hugging
                .foregroundColor(Color.black)
                .lineLimit(1)
        }
        .padding(5)
        .background(Color(UIColor("#d8d8d8")))
        .cornerRadius(12)
        .onTapGesture {
            self.collection.onClick?()
        }
    }

    func iconsView() -> some View {
        let icons = self.collection.hierarchyIcons()
        return HStack(spacing: 2) {
            ForEach(icons, id: \.self) { icon in
                Image(uiImage: icon)
            }
        }
    }
}

struct CollectionPillsView: View {
    private let collections: [CollectionPillData]

    init(collections: [CollectionPillData]) {
        self.collections = collections
    }

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack {
                ForEach(self.collections, id: \.self.id) { collection in
                    CollectionPillView(collection: collection)
                }
            }
        }
        .introspectScrollView { scrollView in
            scrollView.bounces = false
        }
    }
}

class CollectionPillData {
    let id: String
    let name: String
    let icon: UIImage?
    let parent: CollectionPillData?
    let onClick: (() -> Void)?

    init(id: String, name: String, icon: UIImage? = nil, parent: CollectionPillData? = nil, onClick: (() -> Void)? = nil) {
        self.id = id
        self.name = name
        self.icon = icon
        self.parent = parent
        self.onClick = onClick
    }

    func finalIcon() -> UIImage {
        if let icon = self.icon {
            return icon
        } else {
            return ObjectsIcon.icon(forFontAwesome: .folderOpen, color: .black, dimension: 14)
        }
    }

    func hierarchyIcons() -> [UIImage] {
        var icons = [self.finalIcon()]
        var parent = self.parent
        while let curParent = parent {
            icons.append(curParent.finalIcon())
            parent = curParent.parent
        }
        return icons.reversed()
    }

    func lineageLength() -> Int {
        if let length = self.parent?.lineageLength() {
            return length + 1
        } else {
            return 0
        }
    }
}

struct CollectionPills_Previews: PreviewProvider {
    static var previews: some View {
        VStack {
            CollectionPillView(collection: CollectionPillData(id: "col1", name: "Col1"))
            CollectionPillView(collection: CollectionPillData(id: "col1", name: "Col2", icon: "😬".textToImage(ofSize: CollectionPillView.ICON_SIZE)))
            CollectionPillView(collection: CollectionPillData(id: "col1", name: "Long long long long long text"))
            CollectionPillView(collection: CollectionPillData(id: "col1", name: "Child", icon: "👶".textToImage(ofSize: CollectionPillView.ICON_SIZE), parent: CollectionPillData(id: "col2", name: "Parent", icon: "🤷‍♂️".textToImage(ofSize: CollectionPillView.ICON_SIZE), parent: CollectionPillData(id: "col3", name: "Grand parent", icon: "👴".textToImage(ofSize: CollectionPillView.ICON_SIZE)))))

            Spacer().frame(height: 50)

            CollectionPillsView(collections: [CollectionPillData(id: "col1", name: "Col1"), CollectionPillData(id: "col2", name: "Col2")])

            CollectionPillsView(collections: [
                CollectionPillData(id: "col1", name: "Long text"),
                CollectionPillData(id: "col2", name: "Long long long text", icon: "😬".textToImage(ofSize: CollectionPillView.ICON_SIZE)),
                CollectionPillData(id: "col3", name: "Some child with hierarchy", icon: "👶".textToImage(ofSize: CollectionPillView.ICON_SIZE), parent: CollectionPillData(id: "col2", name: "Parent", icon: "🤷‍♂️".textToImage(ofSize: CollectionPillView.ICON_SIZE), parent: CollectionPillData(id: "col3", name: "Grand parent", icon: "👴".textToImage(ofSize: CollectionPillView.ICON_SIZE)))),
                CollectionPillData(id: "col4", name: "Long long long text", icon: "😬".textToImage(ofSize: CollectionPillView.ICON_SIZE)),
            ])

            CollectionPillView(collection: CollectionPillData(id: "col1", name: "Clickable", onClick: {
                print("Clicked")
            }))
        }
    }
}
