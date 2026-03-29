import type { RequestHandler } from "express";
import type { SnapshotService } from "./snapshot.service.js";
import { success, error } from "../../shared/http/response.js";

export function createSnapshotControllers(service: SnapshotService) {
  const getSnapshot: RequestHandler = async (req, res) => {
    const contractId = req.params.contractId as string;
    const snapshot = await service.getSnapshot(contractId);
    if (!snapshot)
      return error(res, { message: "Snapshot not found", status: 404 });
    success(res, snapshot);
  };

  const getSigners: RequestHandler = async (req, res) => {
    const contractId = req.params.contractId as string;
    const isActive = req.query.active === "true" ? true : req.query.active === "false" ? false : undefined;
    
    const signers = await service.getSigners(contractId, { isActive });
    success(res, signers);
  };

  const getSigner: RequestHandler = async (req, res) => {
    const { contractId, address } = req.params;
    const signer = await service.getSigner(contractId, address);
    if (!signer) {
      return error(res, { message: "Signer not found", status: 404 });
    }
    success(res, signer);
  };

  const getRoles: RequestHandler = async (req, res) => {
    const roles = await service.getRoles(req.params.contractId as string);
    success(res, roles);
  };

  const getStats: RequestHandler = async (req, res) => {
    const stats = await service.getStats(req.params.contractId as string);
    if (!stats)
      return error(res, { message: "Snapshot not found", status: 404 });
    success(res, stats);
  };

  return { getSnapshot, getSigners, getSigner, getRoles, getStats };
}
