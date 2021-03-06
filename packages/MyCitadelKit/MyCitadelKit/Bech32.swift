//
//  RGBHelpers.swift
//  MyCitadelKit
//
//  Created by Maxim Orlovsky on 2/2/21.
//

import Foundation

open class Bech32Info {
    public enum Details {
        case unknown
        case url
        case bcAddress(Invoice)
        case bolt11Invoice(Invoice)
        case lnpbpId
        case lnpbpData
        case lnpbpZData
        case lnbpInvoice(Invoice)
        case rgbSchemaId
        case rgbContractId
        case rgbSchema
        case rgbGenesis
        case rgbConsignment
        case rgb20Asset(RGB20Asset)

        public func name() -> String {
            switch self {
            case .unknown:
                return "Unknown"
            case .url:
                return "URL"
            case .bcAddress(_):
                return "Bitcoin address"
            case .bolt11Invoice(_):
                return "LN BOLT11 invoice"
            case .lnpbpId:
                return "LNPBP-39 id"
            case .lnpbpData:
                return "LNPBP-39 data"
            case .lnpbpZData:
                return "LNPBP-39 compressed data"
            case .lnbpInvoice(_):
                return "LNPBP-38 invoice"
            case .rgbSchemaId:
                return "RGB Schema Id"
            case .rgbContractId:
                return "RGB Contract Id"
            case .rgbSchema:
                return "RGB Schema"
            case .rgbGenesis:
                return "RGB Genesis"
            case .rgbConsignment:
                return "RGB Consignment"
            case .rgb20Asset(_):
                return "RGB20 Asset"
            }
        }
    }
    
    public enum ParseStatus: Int32 {
        case ok = 0
        case hrpErr = 1
        case checksumErr = 2
        case encodingErr = 3
        case payloadErr = 4
        case unsupportedErr = 5
        case internalErr = 6
        case invalidJSON = 0xFFFF
    }
    
    public var isOk: Bool {
        parseStatus == .ok
    }
    public var isBech32m: Bool
    public let parseStatus: ParseStatus
    public let parseReport: String
    public let details: Details
    
    public init(_ bech32: String) {
        let info = lnpbp_bech32_info(bech32)

        isBech32m = info.bech32m

        let jsonString = String(cString: info.details)
        let jsonData = Data(jsonString.utf8)
        let decoder = JSONDecoder();
        print("Parsing JSON Bech32 data: \(jsonString)")

        do {
            switch info.category {
            case BECH32_RGB20_ASSET:
                let assetData = try decoder.decode(RGB20Json.self, from: jsonData)
                details = Details.rgb20Asset(RGB20Asset(withAssetData: assetData, citadelVault: CitadelVault.embedded))
            case BECH32_LNPBP_INVOICE:
                let invoice = try decoder.decode(Invoice.self, from: jsonData)
                details = Details.lnbpInvoice(invoice)
            default: details = Details.unknown
            }

            parseStatus = ParseStatus(rawValue: info.status)!
            parseReport = info.status == 0 ? "Bech32 parsed successfully" : String(cString: info.details)
        } catch {
            details = .unknown
            parseStatus = .invalidJSON
            parseReport = "Unable to recognize details from the provided JSON data"
            print("Bech32 parse error \(error.localizedDescription)")
        }
    }
}
