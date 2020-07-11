//
//  NewNoteViewController
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-29.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit
import SnapKit

class NoteViewController: UIViewController, EntityTraitView {
    private var entityTrait: EntityTrait!
    private var localNote: NoteFull?

    private var headerView: LabelledFieldView!
    private var titleField: MultilineTextField!
    private var richTextEditor: RichTextEditor!

    func loadEntityTrait(_ entityTrait: EntityTrait) {
        self.entityTrait = entityTrait
        self.render()
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.createHeaderView()
        self.createWebView()

        self.headerView.snp.makeConstraints { (make) in
            make.width.equalTo(self.view.snp.width)
            make.centerX.equalTo(self.view.snp.centerX)
            make.top.equalTo(self.view.snp.topMargin)
        }

        self.richTextEditor.view.snp.makeConstraints { (make) in
            make.top.equalTo(self.headerView.snp.bottom)
            make.bottom.equalTo(self.view.snp.bottom)
            make.width.equalTo(self.view.snp.width)
            make.centerX.equalTo(self.view.snp.centerX)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
    }

    fileprivate func createHeaderView() {
        self.titleField = MultilineTextField()
        titleField.onChanged = { [weak self] text -> Void in
            self?.handleTitleChange()
        }
        self.headerView = LabelledFieldView(label: "Title", fieldView: titleField)
        self.view.addSubview(self.headerView)
    }

    fileprivate func createWebView() {
        self.richTextEditor = RichTextEditor(callback: { [weak self] (json) -> Void in
            if let body = json?["content"].string, let note = self?.localNote {
                note.content = body
                self?.saveNote() // we don't care to save everytime since it's already debounced in javascript
            }
        })
        self.addChild(self.richTextEditor)
        self.view.addSubview(self.richTextEditor.view)
        self.richTextEditor.didMove(toParent: self)
        self.richTextEditor.viewDidLoad()
    }

    fileprivate func render() {
        if isViewLoaded, self.localNote == nil, let localNote = self.entityTrait.trait as? NoteFull {
            self.localNote = localNote.clone() as? NoteFull

            let content = localNote.content ?? ""
            self.titleField.text = localNote.title
            
            // we don't override text if text editor has focus, since it will trip cursor
            self.richTextEditor.setContent(content)
        }
    }

    fileprivate func handleTitleChange() {
        self.localNote?.title = self.titleField.text
        self.saveNote()
    }

    fileprivate func saveNote() {
        guard   let serverNote = entityTrait.trait as? NoteFull,
                let localNote = self.localNote
            else { return }
        
        
        if !serverNote.equals(localNote) {
            ExomindDSL.on(entityTrait.entity).mutate.put(localNote).execute()
        }
    }

    override func viewWillDisappear(_ animated: Bool) {
        self.saveNote()
    }

    deinit {
        print("NoteViewController > Deinit")
    }
}

