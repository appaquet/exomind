import classNames from 'classnames';
import React from 'react';
import { SelectedItem, Selection } from '../../objects/entity-list/selection';
import Column from './column';
import { ColumnConfig, ColumnsConfig } from './columns-config';
import './columns.less';

interface IProps {
    config: string;
    onConfigChange: (config: ColumnsConfig) => void;
}

export default class Columns extends React.Component<IProps> {
    render(): React.ReactNode {
        const renderedColumns = this.renderColumns();
        const nbColumns = renderedColumns.length;
        const classes = `columns count-${nbColumns}`;
        return (
            <div className={classes}>
                {renderedColumns}
            </div>
        );
    }

    private renderColumns() {
        const config = this.getConfig();
        return config.parts.map((columnConfig, colId) => {
            let selectionItems: SelectedItem[] = [];
            const nextColumnConfig = config.parts[colId + 1];
            if (nextColumnConfig) {
                if (nextColumnConfig.isEntity) {
                    selectionItems = [SelectedItem.fromEntityId(nextColumnConfig.value)];
                } else if (nextColumnConfig.isTrait) {
                    selectionItems = [SelectedItem.fromEntityTraitId(nextColumnConfig.value, nextColumnConfig.extra)];
                }
            }
            const selection = new Selection(selectionItems);

            const colKey = `column-container-${colId}`;
            const classes = classNames({
                'column-container': true,
                [colKey]: true
            });

            // we allow closing any columns, as long as it is not the last one
            const canClose = colId > 0 || nextColumnConfig;

            return (
                <div className={classes} key={colKey}>
                    <Column
                        columnId={colId}
                        columnConfig={columnConfig}
                        key={columnConfig.value}

                        selection={selection}
                        onSelectionChange={this.handleColumnItemSelect.bind(this, colId)}
                        onClose={canClose ? this.handleColumnClose.bind(this, colId) : null}
                    />
                </div>);
        });
    }

    private getConfig(props?: IProps) {
        props = props || this.props;
        let config = ColumnsConfig.fromString(props.config);
        if (config.empty) {
            config = ColumnsConfig.forInbox();
        }
        return config;
    }

    private handleColumnItemSelect(colId: number, objects: Selection) {
        let columnsConfig = this.getConfig();
        if (objects && !objects.isEmpty) {
            // TODO: support for multiple selections + entity traits
            const firstSel = objects.items[0];

            let columnConfig;
            if (firstSel.traitId) {
                columnConfig = ColumnConfig.forTrait(firstSel.entityId, firstSel.traitId);
            } else {
                columnConfig = ColumnConfig.forEntity(firstSel.entityId);
            }
            columnsConfig = columnsConfig.set(colId + 1, columnConfig);

        } else {
            columnsConfig = columnsConfig.unset(colId + 1);
        }

        this.props.onConfigChange(columnsConfig);
    }

    private handleColumnClose(colId: number) {
        const columnsConfig = this.getConfig().pop(colId);
        this.props.onConfigChange(columnsConfig);
    }
}

