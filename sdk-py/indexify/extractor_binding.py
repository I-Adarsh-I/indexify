from dataclasses import dataclass

@dataclass
class ExtractorBinding:
    extractor: str
    name: str
    content_source: str
    filters: dict
    input_params: dict

    def __repr__(self) -> str:
        return f"ExtractorBinding(name={self.name} extractor={self.extractor})"

    def __str__(self) -> str:
        return self.__repr__()

    @classmethod
    def from_dict(cls, json: dict):
        return ExtractorBinding(**json)

