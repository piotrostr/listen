from typing import Optional, Dict, Any, List
from datetime import datetime
from pydantic import BaseModel


class Message(BaseModel):
    role: str
    content: str


class SearchFilters(BaseModel):
    userId: Optional[str] = None
    agentId: Optional[str] = None
    runId: Optional[str] = None


class Entity(BaseModel):
    userId: Optional[str] = None
    agentId: Optional[str] = None
    runId: Optional[str] = None


class AddMemoryOptions(Entity):
    metadata: Optional[Dict[str, Any]] = None
    filters: Optional[SearchFilters] = None
    infer: Optional[bool] = None


class SearchMemoryOptions(Entity):
    limit: Optional[int] = None
    filters: Optional[SearchFilters] = None


class GetAllMemoryOptions(Entity):
    limit: Optional[int] = None


class DeleteAllMemoryOptions(Entity):
    pass


class AddMemory(BaseModel):
    messages: List[Message]
    config: AddMemoryOptions


class MemoryItem(BaseModel):
    id: str
    memory: str
    hash: Optional[str] = None
    createdAt: Optional[str] = None
    updatedAt: Optional[str] = None
    score: Optional[float] = None
    metadata: Optional[Dict[str, Any]] = None


class GraphMemoryResult(BaseModel):
    deleted_entities: List[Any]
    added_entities: List[Any]
    relations: Optional[List[Any]] = None


class AddMemoryResult(BaseModel):
    results: List[MemoryItem]
    graph: Optional[GraphMemoryResult] = None 