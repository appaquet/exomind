import classNames from 'classnames';
import { observer } from 'mobx-react';
import React from 'react';
import { SelectedItem, Selection } from '../../objects/entity-list/selection';
import Column from './column';
import { ColumnConfig, ColumnConfigs } from './columns-config';
import './columns.less';

interface IProps {
    config: string;
    onConfigChange: (config: ColumnConfigs) => void;
}

@observer
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
        return config.parts.flatMap((columnConfig, colId) => {
            if (columnConfig.isMultiple) {
                return [];
            }

            let selectionItems: SelectedItem[] = [];
            const nextColumnConfig = config.parts[colId + 1];
            if (nextColumnConfig) {
                if (nextColumnConfig.isMultiple) {
                    selectionItems = Array.from(nextColumnConfig.values.map((col) => this.configToSelection(col)));
                } else {
                    selectionItems = [this.configToSelection(nextColumnConfig)];
                }
            }
            const selection = new Selection(Array.from(selectionItems.filter((col => !!col))));

            const colClass = `column-container-${colId}`;
            const classes = classNames({
                'column-container': true,
                [colClass]: true
            });
            const colKey = this.columnKey(columnConfig);

            // we allow closing any columns, as long as it is not the last one
            const canClose = colId > 0 || nextColumnConfig;

            return [(
                <div className={classes} key={colKey}>
                    <Column
                        columnId={colId}
                        columnConfig={columnConfig}
                        key={colKey}

                        selection={selection}
                        onSelectionChange={(selection) => this.handleColumnItemSelect(colId, selection)}
                        onClose={canClose ? () => this.handleColumnClose(colId) : null}
                    />
                </div>
            )];
        });
    }

    private columnKey(config: ColumnConfig): string {
        if (config.isSearch) {
            // prevent recreating component for every keystroke
            return 'search';
        } else {
            return config.toString();
        }
    }

    private configToSelection(config: ColumnConfig): SelectedItem {
        if (config.isEntity) {
            return SelectedItem.fromEntityId(config.first);
        } else if (config.isTrait) {
            return SelectedItem.fromEntityTraitId(config.first, config.second);
        }
    }

    private getConfig(props?: IProps): ColumnConfigs {
        props = props || this.props;
        let config = ColumnConfigs.fromString(props.config);
        if (config.empty) {
            config = ColumnConfigs.forInbox();
        }
        return config;
    }

    private handleColumnItemSelect(colId: number, selection: Selection) {
        let columnsConfig = this.getConfig();
        if (selection && !selection.isEmpty) {
            const selectionConfigs = Array.from(selection.items.map((item) => {
                if (item.traitId) {
                    return ColumnConfig.forTrait(item.entityId, item.traitId);
                } else {
                    return ColumnConfig.forEntity(item.entityId);
                }
            }));

            let selectionConfig: ColumnConfig;
            if (selectionConfigs.length == 1) {
                selectionConfig = selectionConfigs[0];
            } else if (selectionConfigs.length > 1) {
                selectionConfig = ColumnConfig.forMultiple(selectionConfigs);
            }

            if (selectionConfig) {
                columnsConfig = columnsConfig.set(colId + 1, selectionConfig);
            }

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

