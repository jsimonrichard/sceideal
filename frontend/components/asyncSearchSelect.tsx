import classNames from "classnames";
import { drop } from "lodash";
import { Key, useEffect, useRef, useState } from "react";
import { useDebounce, useHoverDirty } from "react-use";

interface AsyncSearchSelectProps<V> {
  excludeKeys?: Key[];
  onSelect: (value: V) => void;
  getList: () => Promise<V[]>;
  getKey: (value: V) => Key;
  getName: (value: V) => string;
  placeholder?: string;
}

export default function AsyncSearchSelect<V>({
  excludeKeys = [],
  onSelect,
  getList,
  getKey,
  getName,
  placeholder,
}: AsyncSearchSelectProps<V>) {
  const [list, setList] = useState<V[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    getList()
      .then((list) => {
        setList(list);
        setIsLoading(false);
      })
      .catch((error) => {
        setError(error as Error);
        setIsLoading(false);
      });
  }, []);

  const [searchString, setSearchString] = useState("");
  const [searchResults, setSearchResults] = useState<V[]>([]);
  useDebounce(
    () => {
      setSearchResults(
        list.filter(
          (item) =>
            !excludeKeys.includes(getKey(item)) &&
            getName(item).toLowerCase().includes(searchString.toLowerCase())
        )
      );
    },
    500,
    [searchString]
  );

  // Arrow key item selection state
  const [selectedListIndex, setSelectedListIndex] = useState(-1);
  const dropdownMenuRef = useRef<HTMLDivElement>(null);
  const dropdownMenuIsHovered = useHoverDirty(dropdownMenuRef);
  const [inputFocused, setInputFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (!dropdownMenuIsHovered && inputFocused && searchResults.length > 0) {
      setSelectedListIndex(0);
    } else {
      setSelectedListIndex(-1);
    }
  }, [dropdownMenuIsHovered, inputFocused, searchResults]);

  return (
    <div
      className={classNames({
        dropdown: true,
        "is-active": inputFocused || dropdownMenuIsHovered,
      })}
    >
      <div className="dropdown-trigger">
        <input
          type="text"
          placeholder={placeholder}
          className="input"
          value={searchString}
          ref={inputRef}
          onChange={(e) => setSearchString(e.currentTarget.value)}
          onFocus={() => setInputFocused(true)}
          onBlur={() => setInputFocused(false)}
          onKeyDown={(e) => {
            if (e.key == "ArrowDown") {
              setSelectedListIndex((id) => (id + 1) % list!.length);
            } else if (e.key == "ArrowUp") {
              setSelectedListIndex((id) => (id - 1) % list!.length);
            } else if (e.key == "Enter") {
              if (selectedListIndex != -1) {
                onSelect(list![selectedListIndex]);
                setSearchString("");
              }
            } else if (e.key == "Escape") {
              inputRef.current?.blur();
            }
          }}
        />
      </div>
      <div className="dropdown-menu" role="menu" ref={dropdownMenuRef}>
        <div className="dropdown-content">
          {(() => {
            if (isLoading) {
              return (
                <div className="dropdown-item">
                  <div className="loader-row"></div>
                </div>
              );
            } else if (error !== null) {
              return (
                <p className="dropdown-item has-text-danger">{error.message}</p>
              );
            } else if (searchResults.length == 0) {
              return <p className="dropdown-item">No results found.</p>;
            } else {
              return searchResults.map((item, i) => (
                <a
                  key={getKey(item)}
                  className={classNames({
                    "dropdown-item": true,
                    "is-link": true,
                    "dropdown-psuedo-hover": i == selectedListIndex,
                  })}
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    onSelect(item);
                    setSearchString("");
                  }}
                >
                  {getName(item)}
                </a>
              ));
            }
          })()}
        </div>
      </div>
    </div>
  );
}
